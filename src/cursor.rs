use std::time::{Duration};

// #[derive(Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32
}

impl Point {
    fn distance_from(&self, point: &Point) -> f32 {
        let dx = self.x - point.x;
        let dy = self.y - point.y;
        (dx*dx+dy*dy).sqrt()
    }
}

#[derive(Default, Clone, Copy)]
pub struct Velocity {
    pub x: f32,
    pub y: f32
}

impl Velocity {
    pub fn get_speed(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

pub struct Source {
    pub id: i32,
    pub name: String,
    pub address: String
}

#[derive(Clone, Copy, PartialEq)]
pub enum State {
    Idle,
    Added,
    Accelerating,
    Decelerating,
    Rotating,
    Stopped,
    Removed
}

// #[derive(Clone, Copy)]
pub struct Cursor {
    id: i32,
    time: Duration,
    // source: Source,
    path: Vec<Point>,
    velocity: Velocity,
    acceleration: f32,
    state: State
}

impl Cursor {
    pub fn new(time: Duration, id: i32/*, source: Source*/, position: Point) -> Self {
        Self {
            id,
            path: Vec::from([position]),
            velocity: Velocity::default(),
            acceleration: 0f32,
            time,
            state: State::Added
        }
    }

    pub fn with_movement(mut self, velocity: Velocity, acceleration: f32) -> Self {
        self.velocity = velocity;
        self.acceleration = acceleration;
        self
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn get_time(&self) -> Duration {
        self.time
    }

    pub fn get_x_position(&self) -> f32 {
        self.path.last().unwrap().x
    }

    pub fn get_y_position(&self) -> f32 {
        self.path.last().unwrap().y
    }

    pub fn get_x_velocity(&self) -> f32 {
        self.velocity.x
    }

    pub fn get_y_velocity(&self) -> f32 {
        self.velocity.y
    }

    pub fn get_acceleration(&self) -> f32 {
        self.acceleration
    }

    pub fn get_state(&self) -> State {
        self.state
    }

    pub fn update(&mut self, time: Duration, position: Point) {
        let delta_time = (time - self.time).as_secs_f32();
        let last_position = self.path.last().unwrap();

        let distance = position.distance_from(last_position);
        let delta_x = position.x - last_position.x;
        let delta_y = position.y - last_position.y;

        let last_speed = self.velocity.get_speed();
        let speed = distance / delta_time;

        self.velocity = Velocity{x: delta_x / delta_time, y: delta_y / delta_time};
        self.acceleration = (speed - last_speed) / delta_time;
        self.path.push(position);
        self.time = time;
        
        self.state = if self.acceleration > 0f32 { State::Accelerating } else if self.acceleration < 0f32 { State::Decelerating } else { State::Stopped };
    }

    pub fn update_values(&mut self, time: Duration, position: Point, velocity: Velocity, acceleration: f32) {
        self.time = time;
        self.path.push(position);
        self.velocity = velocity;
        self.acceleration = acceleration;
    }
}