use crate::scanner::utf_string::UtfString;
use core::mem;

pub type Pointer = i32;

pub struct IOnigBinding {
    pub HEAPU8: Vec<u8>,
    pub HEAPU32: Vec<u32>,
}

impl IOnigBinding {
    pub fn new() -> Self {
        IOnigBinding {
            HEAPU8: vec![],
            HEAPU32: vec![],
        }
    }

    // https://users.rust-lang.org/t/how-to-malloc-an-array-in-heap-like-c/27827/34
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=8a124ff92349ac5ca6356bfd832ff1be
    pub fn _malloc<T: Copy>(&self, count: usize) -> *mut T {
        debug_assert!(
            mem::size_of::<T>() > 0,
            "manually allocating a buffer of ZST is a very dangerous idea"
        );
        let mut vec = Vec::<T>::with_capacity(count);
        let ret = vec.as_mut_ptr();
        mem::forget(vec); // avoid dropping the memory
        ret
    }

    pub fn malloc(&self, count: usize) -> *const i32 {
        let mut vec = vec![0; count];
        vec.as_ptr()
    }
}

pub struct OnigScanner {
    pub _onigBinding: IOnigBinding,
    pub _ptr: Pointer,
}

impl OnigScanner {
    pub fn new(pattens: Vec<&str>) -> Self {
        let mut strPtrsArr: Vec<Pointer> = vec![0; pattens.len()];
        let _str_len_arr: Vec<i32> = vec![];
        for i in 0..pattens.len() {
            let pattern = pattens[i].clone();
            let utf_string = UtfString::new(String::from(pattern));
            // strPtrsArr[i] = utf_string.create_string(IOnigBinding::new());
            unsafe {
                let x = utf_string.create_string(IOnigBinding::new());
                strPtrsArr[i] = *(x);
                println!("{:?}, {:?}", strPtrsArr, x);
            }
        }
        println!("{:?}", strPtrsArr);
        OnigScanner {
            _onigBinding: IOnigBinding::new(),
            _ptr: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::scanner::onig_scanner::OnigScanner;

    #[test]
    fn should_init_onig_scanner() {
        OnigScanner::new(vec!["^hello", "workd"]);
        assert!(true)
    }
}