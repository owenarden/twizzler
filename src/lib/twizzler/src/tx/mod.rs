pub trait TxHandle<'obj> {}

pub trait ReadHandle<'obj> {}

impl<'o, T: TxHandle<'o>> ReadHandle<'o> for T {}

pub type TxResult<T, E> = Result<T, TxError<E>>;

pub enum TxError<E> {
    Abort(E),
    Exhausted,
    Immutable,
}

#[repr(transparent)]
pub struct TxObjectCell<T>(T);

impl<T> TxObjectCell<T> {
    pub fn as_ref<'a>(&'a self, tx: impl TxHandle<'a>) -> &T {
        todo!()
    }

    pub fn read<'a>(&'a self, rh: impl ReadHandle<'a>) -> T {
        todo!()
    }

    pub fn as_mut<'a>(&'a self, tx: impl TxHandle<'a>) -> &mut T {
        todo!()
    }

    pub fn write<'a>(&'a self, tx: impl TxHandle<'a>, data: T) {
        todo!()
    }
}
