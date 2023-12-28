use syn::visit::Visit;
use syn::{ExprPath, Ident};

pub(crate) struct CCInterviewInfo<'ast> {
    pub cc_id: ExprPath,
    pub interview_fn: &'ast Ident,
}

pub(crate) struct CCInterviewInfoExtractor<'ast> {
    pub interview: Option<CCInterviewInfo<'ast>>,
}

impl<'ast> Visit<'ast> for CCInterviewInfoExtractor<'ast> {
    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        // We're looking for functions with an #[interview(...)] attribute
        let Some(interview_attr) = i.attrs.iter().find(|a| {
            if a.path().get_ident().map(|i| i.to_string()) == Some("interview".to_string()) {
                a.meta.require_list().is_ok()
            } else {
                false
            }
        }) else {
            return;
        };
        let cc_id_tokens = &interview_attr.meta.require_list().unwrap().tokens;
        let cc_id = syn::parse2::<ExprPath>(cc_id_tokens.clone()).unwrap();

        // ...that are async
        if i.sig.asyncness.is_none() {
            return;
        }

        // FIXME: Raise a compile error if there are multiple interview functions in a module
        self.interview = Some(CCInterviewInfo {
            cc_id,
            interview_fn: &i.sig.ident,
        });
    }
}
