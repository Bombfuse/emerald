use nalgebra::{Vector2, Point2};
use rapier2d::{parry::query::Ray, prelude::{ColliderHandle, InteractionGroups}};


/// # Parameters
/// - `ray`: the ray to cast.
/// - `max_toi`: the maximum time-of-impact that can be reported by this cast. This effectively
///   limits the length of the ray to `ray.dir.norm() * max_toi`. Use `Real::MAX` for an unbounded ray.
/// - `solid`: if this is `true` an impact at time 0.0 (i.e. at the ray origin) is returned if
///            it starts inside of a shape. If this `false` then the ray will hit the shape's boundary
///            even if its starts inside of it.
/// - `query_groups`: the interaction groups which will be tested against the collider's `contact_group`
///                   to determine if it should be taken into account by this query.
/// - `filter`: a more fine-grained filter. A collider is taken into account by this query if
///             its `contact_group` is compatible with the `query_groups`, and if this `filter`
///             is either `None` or returns `true`.
#[derive(Clone)]
pub struct RayCastQuery<'a> {
    pub ray: Ray,
    pub interaction_groups: InteractionGroups,
    pub max_toi: f32,
    pub filter: Option<&'a dyn Fn(ColliderHandle) -> bool>,
    pub solid: bool,
}
impl<'a> Default for RayCastQuery<'a> {
    fn default() -> RayCastQuery<'a> {
        RayCastQuery {
            ray: Ray::new(Point2::new(0.0, 0.0), Vector2::new(0.0, 0.0)),
            max_toi: 4.0,
            filter: None,
            solid: true,
            interaction_groups: InteractionGroups::all(),
        }
    }
}