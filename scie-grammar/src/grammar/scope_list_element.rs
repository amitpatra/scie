#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScopeListElement {
    pub parent: Option<Box<ScopeListElement>>,
    pub scope: String,
    // pub metadata: i32,
}

impl ScopeListElement {
    pub fn new(parent: Option<Box<ScopeListElement>>, scope: String) -> Self {
        ScopeListElement { parent, scope }
    }

    pub fn generate_scopes(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];

        let mut scope_list = self;
        let mut is_scope_list_none = false;
        while !is_scope_list_none {
            result.push(scope_list.scope.clone());
            match &scope_list.parent {
                None => is_scope_list_none = true,
                Some(scope_value) => {
                    scope_list = &*scope_value;
                }
            }
        }

        result.reverse();
        return result;
    }

    pub fn _push(origin_target: ScopeListElement, scopes: Vec<String>) -> ScopeListElement {
        let mut target = origin_target.clone();
        for scope in scopes {
            target = ScopeListElement::new(Some(Box::new(target)), scope);
        }

        target
    }

    pub fn push(&self, scope: Option<String>) -> ScopeListElement {
        if scope.is_none() {
            return self.clone();
        }

        let scope_name = scope.clone().unwrap();
        return match scope.iter().position(|s| s == " ") {
            None => ScopeListElement::_push(self.clone(), vec![scope_name]),
            Some(_) => {
                panic!("todo: ScopeListElement push");
                // self.clone()
            }
        };
    }
}

impl Default for ScopeListElement {
    fn default() -> Self {
        ScopeListElement {
            parent: None,
            scope: "".to_string(),
        }
    }
}
