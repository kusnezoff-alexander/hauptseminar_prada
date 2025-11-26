use crate::opt_extractor::OptCostFunction;
use crate::prada::architecture::PRADAArchitecture;
use eggmock::egg::{Analysis, EClass, Id, Language};
use eggmock::{EggIdToSignal, Mig, MigLanguage, Network, NetworkLanguage, Signal};
use std::cmp::{max, Ordering};
use std::iter;
use std::rc::Rc;

pub struct CompilingCostFunction<'a> {
    pub architecture: &'a PRADAArchitecture,
}

#[derive(Debug)]
pub struct CompilingCost {
    program_cost: usize,
}

impl<A: Analysis<MigLanguage>> OptCostFunction<MigLanguage, A> for CompilingCostFunction<'_> {
    type Cost = Rc<CompilingCost>;

    fn cost<C>(
        &mut self,
        eclass: &EClass<MigLanguage, A::Data>,
        enode: &MigLanguage,
        mut costs: C,
    ) -> Option<Self::Cost>
    where
        C: FnMut(Id) -> Self::Cost,
    {
        // detect self-cycles, other cycles will be detected by compiling, which will result in an
        // error
        if enode.children().contains(&eclass.id) {
            return None;
        }
        let root = enode.clone();
        let cost = match enode {
            MigLanguage::False | MigLanguage::Input(_) => CompilingCost::leaf(root),
            MigLanguage::Not(id) => {
                let cost = costs(*id);

                CompilingCost::with_children(
                    self.architecture,
                    root,
                    iter::once((*id, cost)),
                )?
            }
            MigLanguage::Maj(children) => CompilingCost::with_children(
                self.architecture,
                root,
                children.map(|id| (id, costs(id))),
            )?,
        };
        Some(Rc::new(cost))
    }
}

impl CompilingCost {
    pub fn leaf(root: MigLanguage) -> Self {
        Self {
            program_cost: 0,
        }
    }
    pub fn with_children(
        architecture: &PRADAArchitecture,
        root: MigLanguage,
        child_costs: impl IntoIterator<Item = (Id, Rc<CompilingCost>)>,
    ) -> Option<Self> {
        // let child_graphs = child_costs
        //     .into_iter()
        //     .map(|(id, cost)| cost.collapsed_graph(id));
        // let partial_graph = StackedPartialGraph::new(root, child_graphs);
        // let program_cost = match compile(architecture, &partial_graph.with_backward_edges()) {
        //     Err(_) => return None,
        //     Ok(program) => program.instructions.len(),
        // };
        // Self {
        //     partial: RefCell::new(Either::Left(partial_graph)),
        //     not_nesting,
        //     program_cost,
        // }
        // .into()
        println!("TODO: compiling cost");
        // None
        Some(CompilingCost{program_cost: 1})
    }
}

impl PartialEq for CompilingCost {
    fn eq(&self, other: &Self) -> bool {
        self.program_cost.eq(&other.program_cost)
    }
}

impl PartialOrd for CompilingCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.program_cost.partial_cmp(&other.program_cost)
    }
}
