use anymap::AnyMap;

/// AppContext Is Used for Managing state in your Feather Application  
/// **Context Should only used with objects like Connections and Custom Structs!**
#[derive(Debug)]
pub struct AppContext {
    inner: AnyMap,
}
impl AppContext {
    /// Crate Only Method
    pub(crate) fn new() -> Self {
        Self {
            inner: AnyMap::new(),
        }
    }
    /// Used the Read the State from the Context you can use the turbofish syntax to access objects  
    /// Like this:
    /// ```rust,no_run
    /// let db = ctx.read_state::<Connection>();
    /// ```
    pub fn get_state<T: 'static>(&self) -> Option<&T> {
        self.inner.get::<T>()
    }
    /// Used the Read the State from the Context you can use the turbofish syntax to access objects  
    /// Like this:  
    /// ```rust,no_run
    /// let db = ctx.read_state::<Connection>();
    /// ```  
    /// Returns a Mutable referance to the object
    pub fn get_mut_state<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.get_mut::<T>()
    }
    /// Used to Capture a Object as a State Inside the Context  
    /// That object Ownership now gets transfered to the context  
    /// This method is very useful when using database connections and file accesses
    pub fn set_state<T: 'static>(&mut self, val: T) {
        self.inner.insert::<T>(val);
    }
}
