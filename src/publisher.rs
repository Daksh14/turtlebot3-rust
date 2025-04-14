use r2r::geometry_msgs::msg::Twist;
use r2r::Publisher as RosPublisher;
use r2r::QosProfile;

#[derive(Clone)]
pub struct TwistPublisher(RosPublisher<Twist>);

unsafe impl Send for TwistPublisher {}

impl TwistPublisher {
    pub fn new(nav_node: crate::Node) -> Self {
        let mut lock = nav_node.lock().expect("Failed to lock nav_node");

        let publisher = lock
            .create_publisher::<Twist>("/cmd_vel", QosProfile::default())
            .unwrap();

        TwistPublisher(publisher)
    }

    pub fn publish(&self, msg: &Twist) -> r2r::Result<()> {
        self.0.publish(msg)
    }
}
