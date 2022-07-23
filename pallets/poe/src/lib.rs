#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

// 定义功能模块
#[frame_support::pallet]
pub mod pallet {
  use frame_support::{
    dispatch::DispatchResultWithPostInfo,
    pallet_prelude::*
  };
  use frame_system::pallet_prelude::*;
  use sp_std::vec::Vec;

  // 定义配置接口
  #[pallet::config]
  pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
  }

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  // 定义存储单元
  #[pallet::storage]
  #[pallet::getter(fn proofs)]
  pub type Proofs<T: Config> = StorageMap<
    _,
    Blake2_128Concat,
    Vec<u8>,
    (T::AccountId, T::BlockNumber)
  >;

  // 定义Event事件
  #[pallet::event]
  #[pallet::generate_deposit(pub(super) fn deposit_event)]
  pub enum Event<T: Config> {
    ClaimCreated(T::AccountId, Vec<u8>),
    ClaimRevoked(T::AccountId, Vec<u8>),
    ClaimTransfer(T::AccountId, Vec<u8>, T::AccountId),
  }

  // 定义Error信息
  #[pallet::error]
  pub enum Error<T> {
    ProofAlreadyExist,
    ClaimNotExist,
    NotClaimOwner,
  }

  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

  // 可调用函数
  #[pallet::call]
  impl<T: Config> Pallet<T> {
    // 创建存证函数
    #[pallet::weight(0)]
    pub fn create_claim(
      origin: OriginFor<T>,
      claim: Vec<u8>
    ) -> DispatchResultWithPostInfo {
      // 校验并获取AccountId
      let sender = ensure_signed(origin)?;
      // 校验存证存不存在
      ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);
      // 存储存证
      Proofs::<T>::insert(
        &claim,
        (sender.clone(), frame_system::Pallet::<T>::block_number())
      );
      // 触发事件
      Self::deposit_event(Event::ClaimCreated(sender, claim));
      Ok(().into())
    }

    // 吊销存证函数
    #[pallet::weight(0)]
    pub fn revoke_claim(
      origin: OriginFor<T>,
      claim: Vec<u8>
    ) -> DispatchResultWithPostInfo {
      // 校验并获取AccountId
      let sender = ensure_signed(origin)?;

      // 获取存证,不存在报错
      let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

      // 校验操作用户是否是存证发送方
      ensure!(owner == sender, Error::<T>::NotClaimOwner);

      // 删除存证
      Proofs::<T>::remove(&claim);

      // 触发事件
      Self::deposit_event(Event::ClaimRevoked(sender, claim));
      Ok(().into())
    }

    // 转移存证函数
    #[pallet::weight(0)]
    pub fn transfer_claim(
      origin: OriginFor<T>,
      claim: Vec<u8>,
      recipient: T::AccountId
    ) -> DispatchResultWithPostInfo {
      // 校验并获取AccountId
      let sender = ensure_signed(origin)?;

      // 获取存证,不存在报错
      let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

      // 校验操作用户是否是存证发送方
      ensure!(owner == sender, Error::<T>::NotClaimOwner);

      // 转移存证
      Proofs::<T>::mutate(
        &claim,
        |value|{
          value.as_mut().unwrap().0 = recipient.clone();
          value.as_mut().unwrap().1 = frame_system::Pallet::<T>::block_number();
        }
      );

      // 触发事件
      Self::deposit_event(Event::ClaimTransfer(sender, claim, recipient));
      Ok(().into())
    }
  }
}