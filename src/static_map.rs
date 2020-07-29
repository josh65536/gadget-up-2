use fnv::FnvHashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Index;

/// A map that must be initialized with some arguments 1 time,
/// and then can be used without whose arguments whenever.
/// This is intended to be used for small keys for now
pub struct StaticMap<K, V, F, A>
where
    K: Hash + Copy + Eq,
    F: FnOnce(A) -> FnvHashMap<K, V>,
{
    map: FnvHashMap<K, V>,
    init: Option<F>,
    _marker: PhantomData<fn(A)>,
}

impl<K, V, F, A> StaticMap<K, V, F, A>
where
    K: Hash + Copy + Eq,
    F: FnOnce(A) -> FnvHashMap<K, V>,
{
    /// Creates a new static map with an init function
    pub fn new(init: F) -> Self {
        Self {
            map: FnvHashMap::default(),
            init: Some(init),
            _marker: PhantomData,
        }
    }
}

// higher-kinded types when

impl<K, V, F> StaticMap<K, V, F, ()>
where
    K: Hash + Copy + Eq,
    F: FnOnce(()) -> FnvHashMap<K, V>,
{
    /// Initializes the static map with args.
    /// Call this before using the map.
    pub fn init(&mut self, args: ()) {
        self.map = self
            .init
            .take()
            .expect("Attempted to initialize a static map more than once")(args);
    }
}

impl<'a, K, V, F, RefA> StaticMap<K, V, F, &'a RefA>
where
    K: Hash + Copy + Eq,
    for<'b> F: FnOnce(&'b RefA) -> FnvHashMap<K, V>,
{
    /// Initializes the static map with args.
    /// Call this before using the map.
    pub fn init<'b>(&mut self, args: &'b RefA) {
        self.map = self
            .init
            .take()
            .expect("Attempted to initialize a static map more than once")(args);
    }
}

impl<K, V, F, A> Index<K> for StaticMap<K, V, F, A>
where
    K: Hash + Copy + Eq,
    F: FnOnce(A) -> FnvHashMap<K, V>,
{
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        assert!(
            self.init.is_none(),
            "Initialize the static map before using it"
        );
        &self.map[&index]
    }
}
