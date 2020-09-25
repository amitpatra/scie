/*---------------------------------------------------------
 * Copyright (C) Microsoft Corporation. All rights reserved.
 *--------------------------------------------------------*/

#include "stdlib.h"
#include "string.h"
#include "oniguruma/src/oniguruma.h"
#include <stdbool.h>


typedef struct OnigRegExp_ {
    unsigned char *strData;
    int strLength;
    regex_t *regex;
    OnigRegion *region;
    bool hasGAnchor;
    int lastSearchStrCacheId;
    int lastSearchPosition;
    bool lastSearchMatched;
} OnigRegExp;

typedef struct OnigScanner_ {
    OnigRegSet *rset;
    OnigRegExp **regexes;
    int count;
} OnigScanner;

int lastOnigStatus = 0;
OnigErrorInfo lastOnigErrorInfo;

char *getLastOnigError() {
    static char s[ONIG_MAX_ERROR_MESSAGE_LEN];
    onig_error_code_to_str((UChar *) s, lastOnigStatus, &lastOnigErrorInfo);
    return s;
}

#define MAX_REGIONS 1000

long encodeOnigRegion(OnigRegion *result, int index) {
    static int encodedResult[2 * (1 + MAX_REGIONS)];
    int i;
    if (result == NULL || result->num_regs > MAX_REGIONS) {
        return 0;
    }

    encodedResult[0] = index;
    encodedResult[1] = result->num_regs;
    for (i = 0; i < result->num_regs; i++) {
        encodedResult[2 * i + 2] = result->beg[i];
        encodedResult[2 * i + 3] = result->end[i];
    }
    return (long) encodedResult;
}

#pragma region OnigRegExp

bool hasGAnchor(unsigned char *str, int len) {
    int pos;
    for (pos = 0; pos < len; pos++) {
        if (str[pos] == '\\' && pos + 1 < len) {
            if (str[pos + 1] == 'G') {
                return true;
            }
        }
    }
    return false;
}

OnigRegExp *createOnigRegExp(unsigned char *data, int length) {
    OnigRegExp *result;
    regex_t *regex;

//    if(data[0] == '\\') {
//        if (data[1] == 'G') {
//            char c = '\G';
//            *data = c;
//        }
//    }

    lastOnigStatus = onig_new(&regex, data, data + length,
                              ONIG_OPTION_CAPTURE_GROUP, ONIG_ENCODING_UTF8,
                              ONIG_SYNTAX_DEFAULT, &lastOnigErrorInfo);

    if (lastOnigStatus != ONIG_NORMAL) {
        return NULL;
    }

    result = (OnigRegExp *) malloc(sizeof(OnigRegExp));
    result->strLength = length;
    result->strData = (unsigned char *) malloc(length);
    memcpy(result->strData, data, length);
    result->regex = regex;
    result->region = onig_region_new();
    result->hasGAnchor = hasGAnchor(data, length);
    result->lastSearchStrCacheId = 0;
    result->lastSearchPosition = 0;
    result->lastSearchMatched = false;
    return result;
}

void freeOnigRegExp(OnigRegExp **regex_ptr) {
    OnigRegExp *regex = *regex_ptr;
    // regex->regex will be freed separately / as part of the regset
    free(regex->strData);
    regex->strData = NULL;
    onig_region_free(regex->region, 1);
    regex->region = NULL;
    free(regex);
    regex = NULL;
}

OnigRegion *_searchOnigRegExp(OnigRegExp *regex, unsigned char *strData, int strLength, int position) {
    int status;

    status = onig_search(regex->regex, strData, strData + strLength,
                         strData + position, strData + strLength,
                         regex->region, ONIG_OPTION_NONE);

    if (status == ONIG_MISMATCH || status < 0) {
        regex->lastSearchMatched = false;
        return NULL;
    }

    regex->lastSearchMatched = true;
    return regex->region;
}

OnigRegion *searchOnigRegExp(OnigRegExp *regex, int strCacheId, unsigned char *strData, int strLength, int position) {
    if (regex->hasGAnchor) {
        // Should not use caching, because the regular expression
        // targets the current search position (\G)
        return _searchOnigRegExp(regex, strData, strLength, position);
    }

    if (regex->lastSearchStrCacheId == strCacheId && regex->lastSearchPosition <= position) {
        if (!regex->lastSearchMatched) {
            // last time there was no match
            return NULL;
        }
        if (regex->region->beg[0] >= position) {
            // last time there was a match and it occured after position
            return regex->region;
        }
    }

    regex->lastSearchStrCacheId = strCacheId;
    regex->lastSearchPosition = position;
    return _searchOnigRegExp(regex, strData, strLength, position);
}

#pragma endregion

#pragma region OnigScanner

long createOnigScanner(unsigned char **patterns, int *lengths, int count) {
    int i, j;
    OnigRegExp **regexes;
    regex_t **regs;
    OnigRegSet *rset;
    OnigScanner *scanner;

    regexes = (OnigRegExp **) malloc(sizeof(OnigRegExp *) * count);
    regs = (regex_t **) malloc(sizeof(regex_t *) * count);

    for (i = 0; i < count; i++) {
        regexes[i] = createOnigRegExp(patterns[i], lengths[i]);
        regs[i] = regexes[i]->regex;
        if (regexes[i] == NULL) {
            // parsing this regex failed, so clean up all the ones created so far
            for (j = 0; j < i; j++) {
                free(regs[i]);
                freeOnigRegExp(&regexes[i]);
            }
            free(regexes);
            free(regs);
            return 0;
        }
    }

    onig_regset_new(&rset, count, regs);
    free(regs);

    scanner = (OnigScanner *) malloc(sizeof(OnigScanner));
    scanner->rset = rset;
    scanner->regexes = regexes;
    scanner->count = count;
    return (long) scanner;
}


int freeOnigScanner(OnigScanner **scanner_ptr) {
    int i;
    OnigScanner *scanner = *scanner_ptr;
    for (i = 0; i < scanner->count; i++) {
        freeOnigRegExp(&scanner->regexes[i]);
    }
    free(scanner->regexes);
    scanner->regexes = NULL;
    onig_regset_free(scanner->rset);
    scanner->rset = NULL;
    scanner->count = 0;
    free(scanner);
    scanner = NULL;
    return 0;
}


long findNextOnigScannerMatch(OnigScanner *scanner, int strCacheId, unsigned char *strData, int strLength, int position) {
    int bestLocation = 0;
    int bestResultIndex = 0;
    OnigRegion *bestResult = NULL;
    OnigRegion *result;
    int i;
    int location;

    if (strLength < 1000) {
        // for short strings, it is better to use the RegSet API, but for longer strings caching pays off
        bestResultIndex = onig_regset_search(scanner->rset, strData, strData + strLength, strData + position,
                                             strData + strLength,
                                             ONIG_REGSET_POSITION_LEAD, ONIG_OPTION_NONE, &bestLocation);
        if (bestResultIndex < 0) {
            return 0;
        }
        return encodeOnigRegion(onig_regset_get_region(scanner->rset, bestResultIndex), bestResultIndex);
    }

    for (i = 0; i < scanner->count; i++) {
        result = searchOnigRegExp(scanner->regexes[i], strCacheId, strData, strLength, position);
        if (result != NULL && result->num_regs > 0) {
            location = result->beg[0];

            if (bestResult == NULL || location < bestLocation) {
                bestLocation = location;
                bestResult = result;
                bestResultIndex = i;
            }

            if (location == position) {
                break;
            }
        }
    }

    if (bestResult == NULL) {
        return 0;
    }

    return encodeOnigRegion(bestResult, bestResultIndex);
}

#pragma endregion

void testMakefileGAnchorIssue() {
//    unsigned char* patterns[] = {
//            "^(?!\\t)",
//            "\\G",
//            "^\t"
//    };
//    unsigned char** ptr = patterns;
//
//    int lengths[] = {6, 2, 2};
//
//    unsigned char text[26] = "\t$(CC) -o $@ $^ $(CFLAGS)\n";
//
//    long scanner = createOnigScanner(ptr, lengths, 3);
//    findNextOnigScannerMatch(scanner, 0, text, 27, 0);
}
