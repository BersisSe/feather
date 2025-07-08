use anymap::AnyMap;

#[cfg(feature = "jwt")]
use crate::jwt::JwtManager;

/// AppContext Is Used for Managing state in your Feather Application  
/// **Context Should only used with objects like Connections and Custom Structs!**
#[derive(Debug)]
pub struct AppContext {
    inner: AnyMap,
    #[cfg(feature = "jwt")]
    jwt: Option<JwtManager>
}
impl AppContext {
    /// Crate Only Method
    pub(crate) fn new() -> Self {
        Self {
            inner: AnyMap::new(),
            #[cfg(feature = "jwt")]
            jwt: None
        }
    }
    /// Sets the internal jwt Field with a given Manager
    #[cfg(feature = "jwt")]
    pub fn set_jwt(&mut self,jwt: JwtManager) {
        if self.jwt.is_none(){
            self.jwt = Some(jwt)
        }
    }
    /// Used to Access the JwtManager inside.  
    /// ### Panics 
    /// when called before JwtManager is installed
    #[cfg(feature = "jwt")]
    pub fn jwt(&self) -> &JwtManager {
        self.jwt.as_ref().expect("JwtManager has not been set!")
    }
    /// Used the Read State from the Context you can use the turbofish syntax to access objects  
    /// Like this:
    /// ```rust,ignore
    /// let db = ctx.get_state::<Connection>();
    /// ```
    pub fn get_state<T: 'static>(&self) -> Option<&T> {
        self.inner.get::<T>()
    }
    /// Used the Read Mutable State from the Context you can use the turbofish syntax to access objects  
    /// Like this:  
    /// 
    /// ```rust,ignore
    /// let db = ctx.read_state::<Connection>();
    /// ```  
    /// 
    /// Returns a Mutable referance to the object
    pub fn get_state_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.inner.get_mut::<T>()
    }
    /// Used to Capture a Object as a State Inside the Context  
    /// That object Ownership now gets transfered to the context  
    /// This method is very useful when using database connections and file accesses
    pub fn set_state<T: 'static>(&mut self, val: T) {
        self.inner.insert::<T>(val);
    }
}
