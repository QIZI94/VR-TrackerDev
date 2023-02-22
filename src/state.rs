use std::{mem, fmt::Debug};



use bevy_ecs::{prelude::Component};


pub type PreviousStateResult<T, E> = Result<T, E>;

/// State machine that changes it's state through predefined stages,
///  depeding on result passed by successfull(result) and failed(result) functions.
/// When calling State::Default(), will create equivalent to assigning State::None;
/// 
/// 
/// # Examples
///
/// ```
/// let mut state = State<String, String>::new(Ok("Initalizing!".to_owned()));
/// 
/// while !state.is_done(){
/// 	match state{
/// 		State::Start(previous_result) => {
/// 			state.succesfull("Started successfully");
/// 		},
/// 		State::Run(previous_result) => {
/// 			state.failed("Not good at running");
/// 		},
/// 
/// 		State::Stoped(previous_result) => {
/// 			if !state.check_and_propagate_error(){
/// 				// this will not run due to Error result comming from State::Run
/// 				// instead error from State::Run will be propagated to State::Done
/// 				state successfull("Stopped successfully");
/// 			}
/// 		},
/// 		_ => {/* no other states handled here */}
/// 		
/// 	}
/// }
/// 
/// // for checking the results there are two ways to do it:
/// #1 Directly
/// match state{
/// 	State::Done(result) => {
/// 		if result.ok {
/// 			println!("Result: {}", result.unwrap());
/// 		}
/// 		else {
/// 			println!("Error: {}", result.unwrap_err());
/// 		}
/// 	},
/// }
/// // or
/// #2 By immutable reference
/// if(state.has_result()){
/// 	// unwraping optional first since we already checked it has one
/// 	let result = state.get_last_result().unwrap();
/// 	
/// 	if result.ok {
/// 		println!("Result: {}", result.unwrap());
/// 	}
/// 	else {
/// 		println!("Error: {}", result.unwrap_err());
/// 	}
/// ```
pub enum State<T, E>{
	None,
	Start(PreviousStateResult<T, E>),
	Run(PreviousStateResult<T, E>),
	Stop(PreviousStateResult<T, E>),
	Done(PreviousStateResult<T, E>),
}

impl<T, E>  std::default::Default for State<T, E>  {
	fn default() -> Self {
		State::None
	}
}


impl<T, E> State<T, E>{
	/// Returns new State::Start with PreviousStateResult applied.
    /// This provide consistent way toinitialize state.
    /// # Arguments
    ///
    /// * `initial_previous_state` - used for initializing State::Start
    ///
    /// # Examples
    ///
    /// ```
    ///
	/// let state = State<String, String>::new(Ok("Initalizing!".to_owned()));
	/// 
    /// ```
	pub fn new(initial_previous_state: PreviousStateResult<T, E>) -> Self{
		State::Start(initial_previous_state)
	}

	/// Moving from current state to next one with successfult result.
	/// 
	/// # Order and conditions of state change
	/// 
	/// Start -> Run -> Stop -> Done -> Done(circular)
	/// 
    /// # Arguments
    ///
    /// * `ok` - consumes instance of T and pass ownership to next state
    ///
    /// # Examples
    ///
    /// ```
    ///
	/// let state = State<String, String>::new(Ok("Initalizing!".to_owned()));
	/// 
	/// // in State::Start with "Initalizing!" result before
	/// state.successfull("Successfully started");
	/// // in State::Run with "Successfully started" result after
	/// 
    /// ```
	pub fn successfull(&mut self, ok: T) {
		*self = match self{
			State::None => State::None,
			State::Start(_) => State::Run(Ok(ok)),
			State::Run(_) => State::Stop(Ok(ok)),
			State::Stop(_) => State::Done(Ok(ok)),
			// when state is set to  Done it will just move last result into new Done and drop res parameter
			State::Done(_) => State::Done(self.replace(Some(Ok(ok))).unwrap()),
		};
	}
	
	/// Moving from current state to next one with failiure result.
	/// 
	/// # Order and conditions of state change
	/// 
	/// Start -> Stop -> Done -> Done(circular)
	/// or 
	/// Run -> Stop -> Done -> Done(circular)
	/// 
    /// # Arguments
    ///
    /// * `err` - consumes instance of E and pass ownership to next state
    ///
    /// # Examples
    ///
    /// ```
    ///
	/// let mut state = State<String, String>::new(Ok("Initalizing!".to_owned()));
	/// 
	/// // in State::Start with "Initalizing!" result before
	/// state.failed("Failed to start");
	/// // in State::Stop with "Failed to start" result after
	/// 
    /// ```
	pub fn failed(&mut self, err: E) {
		*self = match self{
			State::None => State::None,
			State::Start(_) => State::Stop(Err(err)),
			State::Run(_) => State::Stop(Err(err)),
			State::Stop(_) => State::Done(Err(err)),
			// when state is set to  Done it will just move last result into new Done and drop res parameter
			State::Done(_) => State::Done(self.replace(Some(Err(err))).unwrap()),
		};
	}


	pub fn pass(&mut self, new_previous_result: PreviousStateResult<T, E>) {
		*self = match self{
			State::None => State::None,
			State::Start(_) => State::Run(new_previous_result),
			State::Run(_) => State::Stop(new_previous_result),
			State::Stop(_) => State::Done(new_previous_result),
			// when state is set to  Done it will just move last result into new Done and drop res parameter
			State::Done(_) => State::Done(self.replace(Some(new_previous_result)).unwrap()),
		};
	}

	pub fn skip(&mut self) {
		let current_state = core::mem::replace(self, State::None);
		*self = match current_state{
			State::None => State::None,
			State::Start(previous_result) => {
			    if previous_result.is_ok() {State::Run(previous_result)} else { State::Stop(previous_result)}
			},
			State::Run(previous_result) => State::Stop(previous_result),
			State::Stop(previous_result) => State::Done(previous_result),
			// when state is set to  Done it will just move last result into new Done and drop res parameter
			State::Done(previous_result) => State::Done(previous_result),
		};
	}

	
	/// Restart by forcing to go to Start::State but only from Stop and Done states with last avaliable result.
	/// 
	/// # Order and conditions of state change
	/// 
	/// Only works in Stop or Done state:
	/// Stop -> Start or Done -> Start
    ///
    /// # Examples
    ///
    /// ```
    ///
	/// let mut state = State<String, String>::new(Ok("Initalizing!".to_owned()));
	/// 
	/// // in State::Start with "Initalizing!" result before
	/// state.successfull("Successfully started");
	/// // in State::Run with "Successfully started" result after
	/// 
	/// // in State::Run with "Successfully started" result before
	/// state.failed("Unsuccessfully ran");
	/// // in State::Stop with "Unsuccessfully ran" result after
	/// 
	/// // oh no we run usuccessfully because something got wrongly set in State::Start
	/// // lets restart and let State::Start know about previous failiure
	/// // in State::Stop with "Unsuccessfully ran" result before
	/// state.restart();
	/// // in State::Start with "Unsuccessfully ran" result before
	/// 
    /// ```
	pub fn restart(&mut self) -> bool{
		
		match self{
			State::Stop(_) => {
				let stop_state = mem::replace(self, State::None); 
				match stop_state{
					State::Stop(last_result) => {
						*self = State::Start(last_result);
					},
					_ => {}
				};
				true
			},
			State::Done(_) => {
				let done_state = mem::replace(self, State::None);
				match done_state{
					State::Done(last_result) => {
						*self = State::Start(last_result);
					},
					_ => {}
				};
				true
			},
			_ => {false}
		};
		false
	}
	
	/// Restart by forcing to go to Start::State but only from Stop, Done or None states with new result.
	/// 
	/// # Order and conditions of state change
	/// 
	/// Only works in Stop or Done or None state:
	/// Stop -> Start or Done -> Start or None -> Start
    ///
    /// # Examples
    ///
    /// ```
    ///
	/// let mut state = State<String, String>::new(Ok("Initalizing!".to_owned()));
	/// 
	/// // in State::Start with "Initalizing!" result before
	/// state.successfull("Successfully started");
	/// // in State::Run with "Successfully started" result after
	/// 
	/// // in State::Run with "Successfully started" result before
	/// state.failed("Unsuccessfully ran");
	/// // in State::Stop with "Unsuccessfully ran" result after
	/// 
	/// // oh no we run usuccessfully because something got wrongly set in State::Start
	/// // lets restart and let State::Start know about previous failiure
	/// // in State::Stop with "Unsuccessfully ran" result before
	/// state.restart_with ("Unfortunetully unsucessfully ran");
	/// // in State::Start with "Unfortunetully unsucessfully ran" result before
	/// 
    /// ```
	pub fn restart_with(&mut self, new_previous_result: PreviousStateResult<T, E>) -> bool{
		
		match self{
			State::None => {
				*self = State::Start(new_previous_result);
				true
			},
			State::Stop(_) => {
				*self = State::Start(new_previous_result);
				true
			},
			State::Done(_) => {
				*self = State::Start(new_previous_result);
				true
			},
			_ => {false}
		};
		false
	}

	/// Replaces result of current state with new result while extracting old one and returning it.
	/// This is useful when you want to get result without copying and result is not imporatnt for your State implementation.
	/// 
	/// When replacing State which doesn't contain result, Option::None will be returned.
	/// 
    /// # Arguments
    ///
    /// * `new_previous_result` - optional which will consume new state and used it for current State as PreviousStateResult 
    ///
    /// # Examples
    ///
    /// ```
    ///
	/// let mut state = State<String, String>::new(Ok("Important secret info!".to_owned()));
	/// let secret: String = state.replace(Some(""));
	/// 
    /// ```
	pub fn replace(&mut self, new_previous_result: Option<PreviousStateResult<T, E>>) -> Option<PreviousStateResult<T, E>> {
		if new_previous_result.is_none(){
			return Option::None
		}
		match self{
			State::Start(last_esult) => Some(mem::replace(last_esult, new_previous_result.unwrap())),
			State::Run(last_esult) => Some(mem::replace(last_esult, new_previous_result.unwrap())),
			State::Stop(last_esult) => Some(mem::replace(last_esult, new_previous_result.unwrap())),
			State::Done(last_esult) => Some(mem::replace(last_esult, new_previous_result.unwrap())),
			State::None => Option::None,
		}
		
	}

	/// Returns PreviousStateResult held by current State. When State doesn't contain result Optional::None 
	/// 
	/// When replacing State which doesn't contain result, Option::None will be returned.
	/// 
    ///
    /// # Examples
    ///
    /// ```
    ///
	/// let mut state = State<String, String>::new(Ok("Current result".to_owned()));
	/// // will contain reference to current State's PreviousStateResult
	/// let result: String = state.get_last_result();
	/// 
    /// ```
	pub fn get_last_result(& self) -> Option<&PreviousStateResult<T, E>> {
		match self{
			State::None => Option::None,
			State::Start(last_result) => Some(&last_result),
			State::Run(last_result) => Some(&last_result),
			State::Stop(last_result) => Some(&last_result),
			State::Done(last_result) => Some(&last_result),

		}
	}

	/// Checks if State is State::Done, which is a final state in which State can be after it went through all other stages.
    /// # Examples
    ///
    /// ```
    ///
	/// let mut state = State<String, String>::new(Ok("Initialized!".to_owned()));
	/// 
	/// while !state.is_done() {
	/// 	state.sucessfull("Sucessfull!");
	/// }
	/// // here state is in State::Done
	/// // next action could be restart() or restart_with(new_result)
	/// 
    /// ```
	pub fn is_done(&self) -> bool{
		match self {
			State::Done(_) => return true,
			_ => return false
		}
	}
	/// Lazy way to check if there is any resut that could be extracted or referenced in current State.
    /// # Examples
    ///
    /// ```
    ///
	/// let mut state = State<String, String>new(Ok("Current result".to_owned()));
	/// 
	/// while !state.is_done() {
	/// 	if !state.has_result(){
	/// 		state.restart_with(Ok("New result"));
	/// 	} 
	/// 	else{
	/// 		state.sucessfull("Sucessfull!");
	/// 	}
	/// }
	/// // here state is in State::Done
	/// // next action could be restart() or restart_with(new_result)
	/// 
    /// ```
	pub fn has_result(&self) -> bool{
		match self{
			State::None => false,
			_ => true
		}
	}

	/// Checks for Err() result and when detected, moves to State::Done state with this error.
	/// This is only avaliable in State::Stop when either State::Start or State::Run, passed error via Failed()
	///  function to propagate this error to State::Done.
	/// 
	/// Returns false when error was found and will be propagated.
    /// # Examples
    ///
    /// ```
    ///
	/// let mut state = State<String, String>::new(Ok("Initialized!".to_owned()));
	/// 
	/// state.failed("Something went wrong when starting");
	/// 
	/// match state {
	///		State::Stop(_) => {
	///			if check_and_propagate_error() {
	/// 			state.successfull("Sucessfully finished all tasks");
	/// 		}
	/// 	},
	///	    _ => {}
	/// }
	/// 
	/// // in State::Done
	/// // will contain refference to result with error "Something went wrong when starting"
	/// let result = state.get_last_result();
	/// 
	/// 
    /// ```
	pub fn check_and_propagate_error (&mut self) -> bool {
		if self.has_result() && self.get_last_result().unwrap().is_err() {
			match self{
				State::Stop(_) => {
					let stop_state = mem::replace(self, State::None); 
					match stop_state{
						State::Stop(last_result) => {
							*self = State::Done(last_result);
						},
						_ => {
							return true;
						}
					};
					false
				},
				_ => {true}
			};
		}
		true
	}
}

/// Trait for implementation that has exclusive access to StateWrapper's internal state.
/// 
pub trait StateHandler<T, E> {
	/// Used for andling state and mutating it exclusively.
	/// 
	/// `state` mutable state which can be read from and mutated
	/// 
	fn handle(&mut self, state: &mut State<T, E>);
}

/// Wrapper arond state, which encapsulates access to it from outside.
/// Only mutation of the State are possible via exclusive access by implementing StateHandler trait.
/// With exception of force_stop() function, which should be only used as last resort.
/// 
/// # Examples
/// ```
/// let mut state_wrapper = StateWrapper<String, String>::new(State::new(Ok("Initalizing!".to_owned())));
/// let my_state_handler = MyStateHandler{};
/// while !state_wrapper.is_done(){
/// 	state_wrapper.apply_handler(my_state_handler);
/// 	
/// }
/// 
/// match state{
/// 	State::Done(result) => {
/// 		if result.ok {
/// 			println!("Result: {}", result.unwrap());
/// 		}
/// 		else {
/// 			println!("Error: {}", result.unwrap_err());
/// 		}
/// 	},
/// }
/// ```
#[derive(Default)]
pub struct StateWrapper<T, E>{
	state: State<T, E>
}

impl<T, E> StateWrapper<T, E>{
	/// With exception of force_stop() function, which should be only used as last resort.
	/// # Arguments 
	/// `state` initial state of StateWrapper
	pub fn new(state: State<T, E>) -> Self{
		Self { state: state }
	}
	/// Gives immutable reference to inner state of the wrapper.
	pub fn get_inner_state(&self) -> &State<T, E>{
		&self.state
	}
	/// Equivalent to state.get_last_result()
	pub fn get_last_result(&self) -> Option<&PreviousStateResult<T, E>>{
		self.state.get_last_result()
	}
	/// Equivalent to state.is_done()
	pub fn is_done(&self) -> bool{
		self.state.is_done()
	}
	/// Equivalent to state.has_result()
	pub fn has_result(&self) -> bool{
		self.state.has_result()
	}

	/// In order to make State go to final stage of being State::Done.
	/// This should be only used as last resord when managing multiple StateWrappers,
	///  or for example to iterate and stop when you need to stop all state related alghoritms,
	///  when your application is preparing to be closed.
	/// 
	pub fn force_stop(&mut self) -> bool{
		
		match self.state{
			State::Start(_) => {
				let start_state = mem::replace(&mut self.state, State::None); 
				match start_state{
					State::Start(last_result) => {
						self.state = State::Stop(last_result);
					},
					_ => {}
				};
				true
			},
			State::Run(_) => {
				let run_state = mem::replace(&mut self.state, State::None); 
				match run_state{
					State::Run(last_result) => {
						self.state = State::Stop(last_result);
					},
					_ => {}
				};
				true
			},
			_ => {false}
		};
		false
	}

	/// In order to make State go to final stage of being State::Done.
	/// This should be only used as last resord when managing multiple StateWrappers,
	///  or for example to iterate and stop when you need to stop all state related alghoritms,
	///  when your application is preparing to be closed.
	/// 
	/// `new_previous_state` new state that will be used when reading State::Stop
	/// 
	pub fn force_stop_with(&mut self, new_previous_state: PreviousStateResult<T, E>) -> bool{
		
		match self.state{
			State::Start(_) => {
				self.state = State::Stop(new_previous_state);
				true
			},
			State::Run(_) => {
				self.state = State::Stop(new_previous_state);
				true
			},
			_ => {false}
		};
		false
	}

	/// Applies handler which implements trait StateHandler, 
	/// and calls handle() function with mutable reference to inner state.
	/// # Arguments 
	/// `handler` dynamice dispatch mut reference to struct with StateHandler implementation
	pub fn apply_handler(&mut self, handler: &mut dyn StateHandler<T, E>){
		handler.handle(&mut self.state);
	}

}

/// Component which has public state member as StateWrapper.
/// This can be used for heaving shared state for whole entity.
#[derive(Default, Component)]
pub struct StateComponent<T, E>{
	pub state: StateWrapper<T, E>
}

pub struct StatePrintHandler;
impl<T: Debug, E: Debug> StateHandler<T, E> for StatePrintHandler{
	fn handle(&mut self, state: &mut State<T, E>){
		print_state(state);
	}
}

fn print_result<Tt: Debug, Ee: Debug>(satrt_with: &str, result: &Result<Tt, Ee>){
	match result {
		Ok(ok) => println!("{satrt_with}{ok:?}"),
		Err(err) => println!("{satrt_with}{err:?}")
	}
}

fn print_state<T: Debug, E: Debug>(state: & State<T, E>) {
	match state{
		State::None
			=> println!("None"),
		State::Start(last_result) 
			=> print_result("Starting with: ", last_result),
		State::Run(last_result) 
			=> print_result("Running with: ", last_result),
		State::Stop(last_result) 
			=> print_result("Stopping with: ", last_result),
		State::Done(last_result) 
			=> print_result("Done with:: ", last_result),
	};
}

fn test_state(state: &mut State<String,String>){
	match state{
		State::Start(previous_state) => {
			let is_ok = previous_state.is_ok();
			print_state(state);
			//state.successfull("Started succesful".to_owned());
			
				
			if is_ok{
				state.failed("Started unsucessfully!!!".to_owned());
			}
			else{
				state.successfull("Started sucessfully".to_owned());
			}
			

		},
		State::Run(_) => {
			print_state(state);
			state.successfull("Running succesful".to_owned());
			
		},
		State::Stop(_) => {
			print_state(state);
			if state.check_and_propagate_error(){
				
				
				state.successfull("Stopped succesfully".to_owned());
				
			}
			
		},
		_ => {}

	};

}

pub fn test() {
	let mut state = State::Start(Ok("For the first time".to_owned()));
	

	while !state.is_done() {
		test_state(&mut state);
	}
	print_state(&state);

	state.restart();
	println!("RESTART!!");
	while !state.is_done() {
		test_state(&mut state);
	}
	print_state(&state);

}
