use std::any::{Any, TypeId};
use std::sync::{Arc, RwLock};

pub trait Variant: Any + Sync + Send + 'static {
    fn type_id(&self) -> TypeId;

    fn as_any(&self) -> &dyn Any;

    fn as_mut_any(&mut self) -> &mut dyn Any;

    fn clone_into_union(&self) -> Union;

    fn type_name(&self) -> &str;
}

impl<T: Clone + Send + Sync + Any> Variant for T {
    #[inline(always)]
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    #[inline(always)]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline(always)]
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }

    #[inline(always)]
    fn clone_into_union(&self) -> Union {
        Union::from(self.clone())
    }

    #[inline(always)]
    fn type_name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

impl std::fmt::Debug for Box<dyn Variant> {
    #[inline(always)]
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        let x: &dyn Variant = &**self;
        write!(formatter, "Variant: {:?}", x.as_any().type_id())?;

        Ok(())
    }
}

#[inline(always)]
fn unsafe_try_cast<A: Any, B: Any>(a: A) -> Option<B> {
    if TypeId::of::<B>() == a.type_id() {
        // SAFETY: Just checked we have the right type. We explicitly forget the
        // value immediately after moving out, removing any chance of a destructor
        // running or value otherwise being used again.
        unsafe {
            let b: B = std::ptr::read(&a as *const _ as *const B);
            std::mem::forget(a);
            Some(b)
        }
    } else {
        None
    }
}

pub type SharedString = Arc<String>;

#[derive(Debug)]
pub enum Union {
    Int(i32),
    Float(f32),
    Bool(bool),
    String(Arc<String>),
    Unit(()),
    Type(UnionType),
    Reference(Box<Variable>),
    Variant(Box<dyn Variant>),
}

impl std::fmt::Display for Union {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{}", v)?,
            Self::Float(v) => write!(f, "{}", v)?,
            Self::Bool(v) => write!(f, "{}", v)?,
            Self::String(v) => write!(f, "{}", v)?,
            Self::Reference(v) => write!(f, "{}", v.cloned())?,
            Self::Unit(_) => write!(f, "()")?,
            Self::Type(t) => write!(f, "{}", t)?,
            Self::Variant(v) => write!(f, "variant<{}>", v.as_ref().type_name())?,
        }

        Ok(())
    }
}

impl Clone for Union {
    #[inline(always)]
    fn clone(&self) -> Self {
        match self {
            Self::Int(i) => Self::Int(*i),
            Self::Float(f) => Self::Float(*f),
            Self::Bool(b) => Self::Bool(*b),
            Self::String(s) => Self::String(s.clone()),
            Self::Reference(r) => Self::Reference(Box::new(r.clone_shared())),
            Self::Unit(()) => Self::Unit(()),
            Self::Type(t) => Self::Type(t.clone()),
            Self::Variant(v) => Variant::clone_into_union(&**v),
        }
    }
}

impl Union {
    #[inline(always)]
    pub fn new<T: Variant>(variant: T) -> Self {
        // union
        if TypeId::of::<T>() == TypeId::of::<Self>() {
            return unsafe_try_cast(variant).unwrap();
        }

        // i32
        if variant.as_any().type_id() == TypeId::of::<i32>() {
            return Self::Int(unsafe_try_cast(variant).unwrap());
        }

        // f32
        if variant.as_any().type_id() == TypeId::of::<f32>() {
            return Self::Float(unsafe_try_cast(variant).unwrap());
        }

        // bool
        if variant.as_any().type_id() == TypeId::of::<bool>() {
            return Self::Bool(unsafe_try_cast(variant).unwrap());
        }

        // string
        if variant.as_any().type_id() == TypeId::of::<SharedString>() {
            return Self::String(unsafe_try_cast(variant).unwrap());
        }

        if variant.as_any().type_id() == TypeId::of::<String>() {
            return Self::String(Arc::new(unsafe_try_cast(variant).unwrap()));
        }

        // unit
        if variant.as_any().type_id() == TypeId::of::<()>() {
            return Self::Unit(());
        }

        // type
        if variant.as_any().type_id() == TypeId::of::<UnionType>() {
            return Self::Type(unsafe_try_cast(variant).unwrap());
        }

        // variant
        Self::Variant(Box::new(variant))
    }

    #[inline(always)]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn from<T: Variant>(variant: T) -> Self {
        Self::new(variant)
    }

    #[inline(always)]
    pub fn downcast<T: Variant>(self) -> Option<T> {
        // union
        if TypeId::of::<T>() == TypeId::of::<Union>() {
            return unsafe_try_cast(self);
        }

        // i32
        if TypeId::of::<T>() == TypeId::of::<i32>() {
            return match self {
                Self::Int(v) => unsafe_try_cast(v),
                _ => None,
            };
        }

        // f32
        if TypeId::of::<T>() == TypeId::of::<f32>() {
            return match self {
                Self::Float(v) => unsafe_try_cast(v),
                _ => None,
            };
        }

        // bool
        if TypeId::of::<T>() == TypeId::of::<bool>() {
            return match self {
                Self::Bool(v) => unsafe_try_cast(v),
                _ => None,
            };
        }

        // string
        if TypeId::of::<T>() == TypeId::of::<String>() {
            return match self {
                Self::String(v) => {
                    let string: String = v.to_string();
                    unsafe_try_cast(string)
                }
                _ => None,
            };
        }
        if TypeId::of::<T>() == TypeId::of::<SharedString>() {
            return match self {
                Self::String(v) => unsafe_try_cast(v.clone()),
                _ => None,
            };
        }

        // unit
        if TypeId::of::<T>() == TypeId::of::<()>() {
            return match self {
                Self::Unit(v) => unsafe_try_cast(v),
                _ => None,
            };
        }

        // type
        if TypeId::of::<T>() == TypeId::of::<UnionType>() {
            return match self {
                Self::Type(v) => unsafe_try_cast(v),
                _ => None,
            };
        }

        // variant
        match self {
            Self::Variant(variant) => {
                let b: Option<Box<T>> = unsafe_try_cast(variant);

                b.map(|inner| *inner)
            }
            _ => None,
        }
    }

    #[inline(always)]
    pub fn downcast_ref<T: Variant + 'static>(&self) -> Option<&T> {
        // union
        if TypeId::of::<T>() == TypeId::of::<Union>() {
            return Some(unsafe { &*(self as *const dyn Any as *const T) });
        }

        // i32
        if TypeId::of::<T>() == TypeId::of::<i32>() {
            return match self {
                Self::Int(v) => <dyn Any>::downcast_ref(v),
                _ => None,
            };
        }

        // f32
        if TypeId::of::<T>() == TypeId::of::<f32>() {
            return match self {
                Self::Float(v) => <dyn Any>::downcast_ref(v),
                _ => None,
            };
        }

        // bool
        if TypeId::of::<T>() == TypeId::of::<bool>() {
            return match self {
                Self::Bool(v) => <dyn Any>::downcast_ref(v),
                _ => None,
            };
        }

        // string
        if TypeId::of::<T>() == TypeId::of::<String>() {
            return match self {
                Self::String(v) => {
                    let string: &String = v;
                    <dyn Any>::downcast_ref(string)
                }
                _ => None,
            };
        }
        if TypeId::of::<T>() == TypeId::of::<SharedString>() {
            return match self {
                Self::String(v) => <dyn Any>::downcast_ref(v),
                _ => None,
            };
        }

        // unit
        if TypeId::of::<T>() == TypeId::of::<()>() {
            return match self {
                Self::Unit(v) => <dyn Any>::downcast_ref(v),
                _ => None,
            };
        }

        // type
        if TypeId::of::<T>() == TypeId::of::<UnionType>() {
            return match self {
                Self::Type(v) => <dyn Any>::downcast_ref(v),
                _ => None,
            };
        }

        // variant
        match self {
            Self::Variant(variant) => <dyn Any>::downcast_ref(variant.as_ref().as_any()),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn downcast_mut<T: Variant + 'static>(&mut self) -> Option<&mut T> {
        // union
        if TypeId::of::<T>() == TypeId::of::<Union>() {
            return <dyn Any>::downcast_mut(self);
        }

        // i32
        if TypeId::of::<T>() == TypeId::of::<i32>() {
            return match self {
                Self::Int(v) => <dyn Any>::downcast_mut(v),
                _ => None,
            };
        }

        // f32
        if TypeId::of::<T>() == TypeId::of::<f32>() {
            return match self {
                Self::Float(v) => <dyn Any>::downcast_mut(v),
                _ => None,
            };
        }

        // bool
        if TypeId::of::<T>() == TypeId::of::<bool>() {
            return match self {
                Self::Bool(v) => <dyn Any>::downcast_mut(v),
                _ => None,
            };
        }

        // string
        if TypeId::of::<T>() == TypeId::of::<String>() {
            return match self {
                Self::String(v) => {
                    let string: &mut String = Arc::make_mut(v);
                    <dyn Any>::downcast_mut(string)
                }
                _ => None,
            };
        }

        if TypeId::of::<T>() == TypeId::of::<SharedString>() {
            return match self {
                Self::String(v) => <dyn Any>::downcast_mut(v),
                _ => None,
            };
        }

        // unit
        if TypeId::of::<T>() == TypeId::of::<()>() {
            return match self {
                Self::Unit(v) => <dyn Any>::downcast_mut(v),
                _ => None,
            };
        }

        // type
        if TypeId::of::<T>() == TypeId::of::<UnionType>() {
            return match self {
                Self::Type(v) => <dyn Any>::downcast_mut(v),
                _ => None,
            };
        }

        // variant
        match self {
            Self::Variant(variant) => <dyn Any>::downcast_mut(variant.as_mut().as_mut_any()),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn ty(&self) -> UnionType {
        match self {
            Self::Int(_) => UnionType::Int,
            Self::Float(_) => UnionType::Float,
            Self::Bool(_) => UnionType::Bool,
            Self::String(_) => UnionType::String,
            Self::Reference(value) => UnionType::Reference(Box::new(value.ty())),
            Self::Unit(_) => UnionType::Unit,
            Self::Type(_) => UnionType::Type,
            Self::Variant(variant) => UnionType::Variant(Variant::as_any(&**variant).type_id()),
        }
    }

    #[inline(always)]
    pub fn as_int(&self) -> Option<i32> {
        match self {
            Self::Int(v) => Some(*v),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn as_float(&self) -> Option<f32> {
        match self {
            Self::Float(v) => Some(*v),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

/// Used for checking types
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum UnionType {
    Int,
    Float,
    Bool,
    String,
    Reference(Box<UnionType>),
    Unit,
    Type,
    Variant(TypeId),
    Any,
}

impl std::fmt::Display for UnionType {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Int => write!(f, "i32")?,
            Self::Float => write!(f, "f32")?,
            Self::Bool => write!(f, "bool")?,
            Self::String => write!(f, "str")?,
            Self::Reference(ty) => write!(f, "&{}", ty)?,
            Self::Unit => write!(f, "()")?,
            Self::Type => write!(f, "type")?,
            Self::Variant(type_id) => write!(f, "variant<{:?}>", type_id)?,
            Self::Any => write!(f, "any")?,
        }

        Ok(())
    }
}

impl UnionType {
    #[inline(always)]
    pub fn from<T: Variant>() -> Self {
        if TypeId::of::<T>() == TypeId::of::<Union>() {
            return Self::Any;
        }

        // i32
        if TypeId::of::<T>() == TypeId::of::<i32>() {
            return Self::Int;
        }

        // f32
        if TypeId::of::<T>() == TypeId::of::<f32>() {
            return Self::Float;
        }

        // bool
        if TypeId::of::<T>() == TypeId::of::<bool>() {
            return Self::Bool;
        }

        // string
        if TypeId::of::<T>() == TypeId::of::<String>() {
            return Self::String;
        }

        if TypeId::of::<T>() == TypeId::of::<&str>() {
            return Self::String;
        }

        if TypeId::of::<T>() == TypeId::of::<SharedString>() {
            return Self::String;
        }

        // type
        if TypeId::of::<T>() == TypeId::of::<UnionType>() {
            return Self::Type;
        }

        // unit
        if TypeId::of::<T>() == TypeId::of::<()>() {
            return Self::Unit;
        }

        // type
        if TypeId::of::<T>() == TypeId::of::<UnionType>() {
            return Self::Type;
        }

        // variant
        Self::Variant(TypeId::of::<T>())
    }
}

pub struct Mut<T> {
    lock: Arc<RwLock<Union>>,
    phantom_data: std::marker::PhantomData<T>,
}

impl<T: Variant> Mut<T> {
    #[inline(always)]
    pub fn cloned(&self) -> Option<T>
    where
        T: Clone,
    {
        match self.lock.read() {
            Ok(union) => union.downcast_ref::<T>().cloned(),
            Err(_) => None,
        }
    }

    #[inline(always)]
    pub fn cell(self) -> UnionCell {
        UnionCell::Shared(self.lock)
    }

    #[inline(always)]
    pub fn map<R, F: FnMut(&T) -> R>(&self, mut f: F) -> R {
        match self.lock.read() {
            Ok(union) => match union.downcast_ref::<T>() {
                Some(t) => f(t),
                None => panic!("unreachable, couldn't reference to T in Mut<T>"),
            },
            Err(_) => panic!("Dead lock"),
        }
    }

    #[inline(always)]
    pub fn map_mut<R, F: FnMut(&mut T) -> R>(&mut self, mut f: F) -> R {
        match self.lock.write() {
            Ok(mut union) => match union.downcast_mut::<T>() {
                Some(t) => f(t),
                None => panic!("unreachable, couldn't reference to T in Mut<T>"),
            },
            Err(_) => panic!("Dead lock"),
        }
    }
}

/// Holds either an Owned [`Union`] or a Shared one.
#[derive(Debug)]
pub enum UnionCell {
    Owned(Union),
    Shared(Arc<RwLock<Union>>),
}

impl UnionCell {
    #[inline(always)]
    pub fn new<T: Variant>(union: T) -> Self {
        if TypeId::of::<T>() == TypeId::of::<UnionCell>() {
            unsafe_try_cast(union).unwrap()
        } else {
            Self::Owned(Union::from(union))
        }
    }

    #[inline(always)]
    pub fn into_shared(self) -> Self {
        match self {
            Self::Owned(union) => Self::Shared(Arc::new(RwLock::new(union))),
            Self::Shared(_) => self,
        }
    }

    #[inline(always)]
    pub fn make_shared(&mut self) {
        let union = match self {
            Self::Owned(union) => union.clone(),
            Self::Shared(_) => return,
        };

        *self = Self::Shared(Arc::new(RwLock::new(union)));
    }

    #[inline(always)]
    pub fn get_shared(&mut self) -> Self {
        self.make_shared();

        match self {
            Self::Shared(shared) => Self::Shared(shared.clone()),
            Self::Owned(_) => panic!("Somehow got owned after make shared"),
        }
    }

    #[inline(always)]
    pub fn clone_shared(&self) -> Self {
        match self {
            Self::Shared(lock) => Self::Shared(lock.clone()),
            Self::Owned(union) => Self::Owned(union.clone()),
        }
    }

    #[inline(always)]
    pub fn is_owned(&self) -> bool {
        match self {
            Self::Owned(_) => true,
            Self::Shared(_) => false,
        }
    }

    #[inline(always)]
    pub fn get_lock<T>(&mut self) -> Mut<T> {
        self.make_shared();

        match self {
            Self::Shared(lock) => Mut {
                lock: lock.clone(),
                phantom_data: Default::default(),
            },
            Self::Owned(_) => panic!("Somehow got owned after make shared"),
        }
    }

    #[inline(always)]
    pub fn is_shared(&self) -> bool {
        !self.is_owned()
    }

    #[inline(always)]
    pub fn cloned(&self) -> Union {
        self.map(|v| v.clone())
    }

    #[inline(always)]
    pub fn into_inner(self) -> Union {
        match self {
            Self::Shared(shared) => shared.read().unwrap().clone(),
            Self::Owned(union) => union,
        }
    }

    #[inline(always)]
    pub fn ty(&self) -> UnionType {
        self.map(|v| v.ty())
    }

    #[inline(always)]
    pub fn set(&mut self, union: Union) {
        match self {
            Self::Owned(owned) => *owned = union,
            Self::Shared(shared) => *shared.write().unwrap() = union,
        }
    }

    #[inline(always)]
    pub fn map<T, F: FnMut(&Union) -> T>(&self, mut f: F) -> T {
        match self {
            Self::Owned(union) => f(union),
            Self::Shared(union) => {
                let union = union.read().expect("Dead Lock");

                f(&*union)
            }
        }
    }

    #[inline(always)]
    pub fn map_mut<T, F: FnMut(&mut Union) -> T>(&mut self, mut f: F) -> T {
        match self {
            Self::Owned(union) => f(union),
            Self::Shared(union) => {
                let mut union = union.write().expect("Dead Lock");

                f(&mut *union)
            }
        }
    }
}

impl Clone for UnionCell {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self::Owned(self.cloned())
    }
}

impl From<Union> for UnionCell {
    #[inline(always)]
    fn from(union: Union) -> Self {
        Self::Owned(union)
    }
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub type_specified: bool,
    pub union: UnionCell,
}

impl Variable {
    pub fn new(union: impl Into<UnionCell>, type_specified: bool) -> Self {
        Self {
            type_specified,
            union: union.into(),
        }
    }

    #[inline(always)]
    pub fn specified(union: impl Into<UnionCell>) -> Self {
        Self {
            type_specified: true,
            union: union.into(),
        }
    }

    #[inline(always)]
    pub fn unspecified(union: impl Into<UnionCell>) -> Self {
        Self {
            type_specified: false,
            union: union.into(),
        }
    }

    #[inline(always)]
    pub fn clone_shared(&self) -> Self {
        Self {
            type_specified: self.type_specified.clone(),
            union: self.union.clone_shared(),
        }
    }

    #[inline(always)]
    pub fn get_shared(&mut self) -> Self {
        Self {
            type_specified: self.type_specified.clone(),
            union: self.union.get_shared(),
        }
    }

    #[inline(always)]
    pub fn into_inner(self) -> Union {
        self.union.into_inner()
    }

    #[inline(always)]
    pub fn union_type(&self) -> UnionType {
        self.union.ty()
    }
}

impl std::ops::Deref for Variable {
    type Target = UnionCell;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.union
    }
}

impl std::ops::DerefMut for Variable {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.union
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // i had an issue where the stack would overflow when cloning a union
    #[test]
    fn union_clone() {
        let _ = Union::new(2).clone();
    }

    #[test]
    fn union_conversion() {
        macro_rules! ty {
            ($ty:ty, $ident:ident) => {
                let mut x = Union::new(<$ty>::default());

                match &x {
                    Union::$ident(_) => {}
                    u => panic!("Union assigned wrong type {:?}", u),
                }

                assert_eq!(Some(&<$ty>::default()), x.downcast_ref());
                assert_eq!(Some(&mut <$ty>::default()), x.downcast_mut());
            };
        }

        ty!(i32, Int);
        ty!(f32, Float);
        ty!(bool, Bool);
        ty!((), Unit);

        #[derive(Clone, Debug, PartialEq, Default)]
        struct Foo;

        ty!(Foo, Variant);

        let x = Union::from(3.14);
        let z: Union = x.clone();
        let y = Union::from(z);

        assert_eq!(x.downcast_ref::<f32>(), y.downcast_ref::<f32>());
    }

    #[test]
    fn shared_union_cell() {
        let mut cell = UnionCell::new(Union::from(3));
        let unshared_cell = cell.clone();
        // get shared
        let shared_cell = cell.get_shared();

        // mutate cell
        cell.map_mut(|v| *v.downcast_mut::<i32>().unwrap() += 2);

        assert_eq!(cell.cloned().downcast_ref::<i32>(), Some(&5));
        assert_eq!(shared_cell.cloned().downcast_ref::<i32>(), Some(&5));
        assert_eq!(unshared_cell.cloned().downcast_ref::<i32>(), Some(&3));

        assert_eq!(
            cell.cloned().downcast_ref::<i32>(),
            shared_cell.cloned().downcast_ref::<i32>()
        );
        assert_ne!(
            cell.cloned().downcast_ref::<i32>(),
            unshared_cell.cloned().downcast_ref::<i32>()
        );
    }
}
