//! Query/condition tables: [](https://www.dataorienteddesign.com/dodmain/node7.html#SECTION00710000000000000000) and [](https://www.dataorienteddesign.com/dodmain/node12.html); it's in C but the ideas are similar.
//!
//! A query table asks a question of a table. Our tables in asterism are the identities/syntheses, and the events; ex. for a collision logic the identities could be the positions/sizes/metadata of each collision body, while the events would be contacts.
//!
//! A condition table composes those individual queries together.

/// Builds a query table over each "unit" of a logic.
///
/// kind of weird, I don't like the reallocations but I don't think it's worse than what I'm doing in the engines with building syntheses every frame.
pub trait QueryTable<QueryOver> {
    fn check_predicate(&self, predicate: impl Fn(&QueryOver) -> bool) -> Vec<bool>;
}

pub struct ConditionTables {
    query_output: Vec<Vec<bool>>, // scary to have all these loose tables without types
    conditions: Vec<Condition>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct QueryID(usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ConditionID(usize);

impl ConditionTables {
    pub fn new() -> Self {
        Self {
            query_output: Vec::new(),
            conditions: Vec::new(),
        }
    }

    pub fn add_query(&mut self) -> QueryID {
        let id = QueryID(self.query_output.len());
        self.query_output.push(Vec::new());
        id
    }

    pub fn update_query(&mut self, id: QueryID, output: Vec<bool>) {
        self.query_output[id.0] = output;
    }

    pub fn add_condition(&mut self, compose: Compose) -> ConditionID {
        let id = ConditionID(self.conditions.len());
        self.conditions.push(Condition {
            compose,
            output: Vec::new(),
        });
        self.check_condition(id);
        id
    }

    pub fn check_condition(&mut self, condition: ConditionID) -> &[bool] {
        let compose = &self.conditions[condition.0].compose;
        let output = self.conditions[condition.0].output.clone();
        match self.check(compose, &output) {
            Answer::Once(answer, len) => self.conditions[condition.0]
                .output
                .resize_with(len, || answer),
            Answer::Many(answers) => self.conditions[condition.0].output = answers,
        };
        &self.conditions[condition.0].output
    }

    fn check(&self, compose: &Compose, output: &[bool]) -> Answer {
        match compose {
            Compose::Just(q_id, process) => {
                let output = self.query_output[q_id.0].clone();

                match process {
                    ProcessOutput::ForEach => Answer::Many(output),
                    ProcessOutput::IfAny => {
                        Answer::Once(output.iter().any(|unit| *unit), output.len())
                    }
                    ProcessOutput::IfNone => {
                        Answer::Once(!output.iter().any(|unit| *unit), output.len())
                    }
                    ProcessOutput::IfLength(len, cmp) => Answer::Once(
                        output.iter().filter(|unit| **unit).count().cmp(len) == *cmp,
                        output.len(),
                    ),
                }
            }
            Compose::And(comp_1, comp_2) => {
                let out_1 = self.check(&comp_1, output);
                let out_2 = self.check(&comp_2, output);
                match (out_1, out_2) {
                    (Answer::Once(a1, len1), Answer::Once(a2, len2)) => {
                        let mut compose = Vec::new();
                        compose.resize_with(len1.max(len2), || a1 && a2);
                        Answer::Many(compose)
                    }
                    (Answer::Once(a1, _), Answer::Many(mut a2))
                    | (Answer::Many(mut a2), Answer::Once(a1, _)) => {
                        for a2 in a2.iter_mut() {
                            *a2 = *a2 && a1;
                        }
                        Answer::Many(a2)
                    }
                    (Answer::Many(mut a1), Answer::Many(mut a2)) => {
                        for (a1, a2) in a1.iter_mut().zip(a2.iter_mut()) {
                            *a1 = *a1 && *a2;
                        }
                        Answer::Many(a1)
                    }
                }
            }
            Compose::Or(comp_1, comp_2) => {
                let out_1 = self.check(&comp_1, output);
                let out_2 = self.check(&comp_2, output);
                match (out_1, out_2) {
                    (Answer::Once(a1, len1), Answer::Once(a2, len2)) => {
                        let mut compose = Vec::new();
                        compose.resize_with(len1.max(len2), || a1 || a2);
                        Answer::Many(compose)
                    }
                    (Answer::Once(a1, _), Answer::Many(mut a2))
                    | (Answer::Many(mut a2), Answer::Once(a1, _)) => {
                        for a2 in a2.iter_mut() {
                            *a2 = *a2 || a1;
                        }
                        Answer::Many(a2)
                    }
                    (Answer::Many(mut a1), Answer::Many(mut a2)) => {
                        for (a1, a2) in a1.iter_mut().zip(a2.iter_mut()) {
                            *a1 = *a1 || *a2;
                        }
                        Answer::Many(a1)
                    }
                }
            }
        }
    }
}

/// Possible ways to compose queries. Should probably use `Rc`s instead of `Box`es
///
/// bad joke about how this crate is called "ASTerism"
#[non_exhaustive]
#[derive(Clone)]
pub enum Compose {
    Just(QueryID, ProcessOutput),
    And(Box<Compose>, Box<Compose>),
    Or(Box<Compose>, Box<Compose>),
}

enum Answer {
    Once(bool, usize), // longest length
    Many(Vec<bool>),
}

/// Possible ways to deal with the output of queries
#[non_exhaustive]
#[derive(Clone, Copy)]
pub enum ProcessOutput {
    ForEach,
    IfAny,
    IfNone,
    IfLength(usize, std::cmp::Ordering),
}

pub struct Condition {
    compose: Compose,
    output: Vec<bool>,
}
