use std::{ collections::VecDeque, sync::{Mutex, Condvar} };

pub struct TSDequeue<T> {
	data: Mutex<VecDeque<T>>,
	cv: Condvar
}

impl<T> TSDequeue<T> {
	pub fn new() -> Self {
		Self {
			data: Mutex::new(VecDeque::new()),
			cv: Condvar::new()
		}
	}

	pub fn push(&self, value: T) {
		let mut data = self.data.lock().unwrap();
		data.push_back(value);

		self.cv.notify_one();
	}

	pub fn pop(&self) -> T {
		let mut data = self.data.lock().unwrap();

		while data.is_empty() {
			data = self.cv.wait(data).unwrap();
		}
		
		data.pop_front().unwrap()
	}

	pub fn len(&self) -> usize {
		self.data.lock().unwrap().len()
	}

	pub fn is_empty(&self) -> bool {
		self.data.lock().unwrap().is_empty()
	}
}