use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use dynlink::{
    context::Context,
    engines::{Backing, Engine},
};
use monitor_api::TlsTemplateInfo;
use twizzler_object::ObjID;
use twz_rt::monitor::RuntimeThreadControl;

use crate::{compartment::Comp, init::InitDynlinkContext};

pub struct MonitorState {
    pub dynlink: &'static mut Context<Engine>,
    pub(crate) root: String,

    pub comps: HashMap<ObjID, Comp>,
}

impl MonitorState {
    pub(crate) fn new(init: InitDynlinkContext) -> Self {
        Self {
            dynlink: unsafe { init.ctx.as_mut().unwrap() },
            root: init.root,
            comps: Default::default(),
        }
    }

    pub(crate) fn get_nth_library(&self, n: usize) -> Option<&dynlink::library::Library<Backing>> {
        // TODO: this sucks.
        let mut all = vec![];
        // TODO
        let comp_id = self.dynlink.lookup_compartment("monitor")?;
        let root_id = self.dynlink.lookup_library(comp_id, &self.root)?;
        self.dynlink
            .with_bfs(root_id, |lib| all.push(lib.name().to_string()));
        let lib = all
            .get(n)
            // TODO
            .and_then(|x| match self.dynlink.lookup_library(comp_id, &x) {
                Some(l) => Some(l),
                _ => None,
            })?;

        self.dynlink.get_library(lib).ok()
    }

    pub(crate) fn add_comp(&mut self, comp: Comp) {
        let cc = unsafe { comp.get_comp_config().as_ref().unwrap() };
        let template_info = self
            .dynlink
            .get_compartment_mut(comp.compartment_id)
            .unwrap()
            .build_tls_region(RuntimeThreadControl::new(0))
            .unwrap();
        let temp = Box::new(TlsTemplateInfo::from(template_info));
        let temp = Box::leak(temp);
        cc.set_tls_template(temp);
        self.comps.insert(comp.sctx_id, comp);
    }
}

static MONITOR_STATE: OnceLock<Arc<Mutex<MonitorState>>> = OnceLock::new();

pub(crate) fn set_monitor_state(state: Arc<Mutex<MonitorState>>) {
    MONITOR_STATE
        .set(state)
        .unwrap_or_else(|_| panic!("monitor state already set"))
}

pub(crate) fn get_monitor_state() -> &'static Arc<Mutex<MonitorState>> {
    MONITOR_STATE
        .get()
        .unwrap_or_else(|| panic!("failed to get monitor state"))
}
