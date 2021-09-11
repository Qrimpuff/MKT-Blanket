use mkt_data::{Course, MktDatabase};
use yew_agent::{AgentLink, utils::store::{Store, StoreWrapper}};


pub struct DataStore {
    pub data: MktDatabase,
}

impl Store for DataStore {
    type Input = ();
    type Action = ();

    fn new() -> Self {
        let mut data = MktDatabase::new();
        data.courses.insert("test_1".into(), Course::new("Test 1".into()));
        Self { data }
    }

    fn handle_input(&self, _link: AgentLink<StoreWrapper<Self>>, _msg: Self::Input) {
        todo!()
    }

    fn reduce(&mut self, _msg: Self::Action) {
        todo!()
    }
}