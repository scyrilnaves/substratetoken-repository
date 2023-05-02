#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

//Conversion
//https://stackoverflow.com/questions/56081117/how-do-you-convert-between-substrate-specific-types-and-rust-primitive-types
//https://github.com/paritytech/substrate/issues/3560
// Float problems: https://paritytech.github.io/ink-docs/faq/
#[frame_support::pallet]
pub mod pallet {
	use core::convert::TryInto;
	use frame_support::log::info;
	use frame_support::pallet_prelude::*;
	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};
	use frame_support::sp_io::hashing::blake2_128;
	use frame_support::sp_runtime::SaturatedConversion;
	use frame_support::{
		sp_runtime::traits::Hash,
		traits::{tokens::ExistenceRequirement, Currency, Randomness},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Struct for holding  information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Property {
		pub details: u64,
		pub initialprice: i32,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types it depends on.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The Currency handler for the pallet.
		type Currency: Currency<Self::AccountId>;

		/// The type of Randomness we want to specify for this pallet.
		type PropertyRandomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		/// Handles checking whether the Property exists.
		PropertyNotExist,
		/// Handles checking that the Property is owned by the account transferring, buying or setting a price for it.
		NotPropertyOwner,
		/// Ensures that the buying price is greater than the asking price.
		NoAvailableTokens,
		/// Ensures that an account has enough funds to purchase a Property.
		NotEnoughBalance,
		NotAdminAccount,
	}

	// Events.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// AdminAccountAdded
		AdminAccountAdded(T::AccountId),
		// ComissionAccountAdded
		ComissionAccountAdded(T::AccountId),
		/// A new Property was successfully created. \[sender, property_id\]
		Created(T::AccountId, T::Hash),
		/// A Property was successfully deleted. \[sender, property_id\]
		Deleted(T::AccountId, T::Hash),
		// Property was not deleted sucessfully
		Delete_UnSucessfull(T::AccountId, T::Hash),
		/// Propert price was successfully set. \[sender, property_id, new_price\]
		PriceModified(T::AccountId, T::Hash, i32),
		/// A Token was successfully transferred. \[from, to, propertyid\]
		Sold(T::AccountId, T::Hash),
		/// A Token was not successfully transferred. \[from, to, property_id\]
		SellError(T::AccountId, T::Hash),
		/// Tokens were successfully bought. \[buyer,property_id]
		Bought(T::AccountId, T::Hash),
        // Property Detail modified
		DetailModified(T::AccountId, T::Hash),
        // Sell Comission Percent Added
		SellComissionPercentAdded(u32),
		// Buy Comission Percent Added
		BuyComissionPercentAdded(u32),
	}

	// Storage items.
	#[pallet::storage]
	#[pallet::getter(fn adminacc)]
	/// Assign the admin accounts.
	pub(super) type AdminAcc<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn comissionacc)]
	/// Assign the comission account of the plattform.
	pub(super) type ComissionAcc<T: Config> =
		StorageMap<_, Twox64Concat, u32, T::AccountId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn sellcomission)]
	/// Comission to the plattform for a sale
	pub(super) type SellComission<T: Config> = StorageMap<_, Twox64Concat, u32, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn buycomission)]
	/// Comission to the plattform for purchase
	pub(super) type BuyComission<T: Config> = StorageMap<_, Twox64Concat, u32, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn property_cnt)]
	/// Keeps track of the number of Properties in existence.
	pub(super) type PropertyCnt<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn properties)]
	/// Stores a Propertys's Id, Property Detail, PropertyOwnershipId.
	pub(super) type Properties<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, Property, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn propertyprice)]
	/// Stores a Propertys's Id, Property Price.
	pub(super) type PropertyPrice<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, i32, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn primary_property_owners)]
	/// Stores a Propertys's Id, Property Detail, PropertyOwnershipId.
	pub(super) type PrimaryPropertyOwners<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, T::AccountId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn token_circulation)]
	/// Stores the Token Circulation for Each Property
	/// Can be reinitialised during price reset
	pub(super) type TokenCirculation<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, i32, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn token_price)]
	/// Stores the Token price available for Each Property
	/// Can be reinitialised during price reset of the property
	/// PropertyId, Token Price
	pub(super) type TokenPrice<T: Config> = StorageMap<_, Twox64Concat, T::Hash, i32, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn properties_ownership)]
	/// Keeps track of each property ownerhsip details.
	/// (PropertyId,index)(Accountid, TokenBalance)
	/// At index 0, we store the primary ownership
	/// From index 1, we store the new secondary owners
	pub(super) type PropertiesOwnership<T: Config> =
		StorageMap<_, Twox64Concat, (T::Hash, u32), (T::AccountId, u32), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn secondary_owner_index)]
	/// Keeps track of total owners for each property.
	pub(super) type SecondaryOwnerIndex<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn secondary_owners)]
	/// Keeps track of total owners for each property.
	pub(super) type SecondaryOwner<T: Config> =
		StorageMap<_, Twox64Concat, (T::Hash, T::AccountId), u32, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		///Add an admin to the Pallet
		///
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn add_superadmin(origin: OriginFor<T>, admin: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;

			//info!("called2");
			<AdminAcc<T>>::insert(&admin, 1);

			Self::deposit_event(Event::AdminAccountAdded(admin));
			Ok(())
		}

		///Add a comission account to the Plattform
		///
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn add_comissionacount(
			origin: OriginFor<T>,
			comissionacount: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;

			<ComissionAcc<T>>::insert(0, &comissionacount);

			Self::deposit_event(Event::ComissionAccountAdded(comissionacount));
			Ok(())
		}

		///Add a Sell comission percent to the Plattform
		///
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn add_sellcomissionpercent(
			origin: OriginFor<T>,
			comissionpercent: u32,
		) -> DispatchResult {
			ensure_root(origin)?;

			<SellComission<T>>::insert(0, comissionpercent);

			Self::deposit_event(Event::SellComissionPercentAdded(comissionpercent));
			Ok(())
		}

		///Add a Purchase comission percent to the Plattform
		///
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn add_buycomissionpercent(
			origin: OriginFor<T>,
			comissionpercent: u32,
		) -> DispatchResult {
			ensure_root(origin)?;

			<BuyComission<T>>::insert(0, comissionpercent);

			Self::deposit_event(Event::BuyComissionPercentAdded(comissionpercent));
			Ok(())
		}

		////// Create a new unique Property.
		/// The actual Property creation is done in the `mint()` function.
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn create_property(
			origin: OriginFor<T>,
			owner: T::AccountId,
			details: u64,
			price: i32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// Check if account is an admin
			ensure!(<AdminAcc<T>>::contains_key(sender.clone()), Error::<T>::NotAdminAccount);

			let property_id = Self::mint(&owner, details, price)?;

			Self::deposit_event(Event::Created(sender.clone(), property_id));
			Ok(())
		}

		////// Remove a new  Property.
		/// The actual Property deletion is done in the `burn()` function.
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn delete_property(origin: OriginFor<T>, property_id: T::Hash) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			// Check if account is an admin
			ensure!(<AdminAcc<T>>::contains_key(sender.clone()), Error::<T>::NotAdminAccount);

			let result = Self::burn(property_id.clone());

			if (result) {
				Self::deposit_event(Event::Deleted(sender.clone(), property_id));
			} else {
				Self::deposit_event(Event::Delete_UnSucessfull(sender.clone(), property_id));
			}

			Ok(())
		}

		/// Modify the price for a Property.
		///
		/// Updates Property price and updates storage.
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn modify_property_price(
			origin: OriginFor<T>,
			property_id: T::Hash,
			new_price: i32,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Check if account is an admin
			ensure!(<AdminAcc<T>>::contains_key(sender.clone()), Error::<T>::NotAdminAccount);

			// Ensure it is the Property Owner--- Not needed
			//ensure!(<PrimaryPropertyOwners<T>>::get(property_id).unwrap() == sender.clone(),<Error<T>>::NotPropertyOwner);
			let old_price = Self::propertyprice(property_id).unwrap();
			//Update the property price
			<PropertyPrice<T>>::insert(property_id, new_price);
			//Update the token price
			let difference: i32 = new_price - old_price;
			let percent: i32 = (difference / old_price) * 100;
			let newtokenprice: i32 = old_price + (percent * 100);
			<TokenPrice<T>>::insert(property_id, newtokenprice);
			// Deposit a "PriceSet" event.
			Self::deposit_event(Event::PriceModified(sender, property_id, new_price));

			Ok(())
		}

		/// Modify the details for a Property.
		///
		/// Updates Property details and updates storage.
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn modify_property_details(
			origin: OriginFor<T>,
			property_id: T::Hash,
			detail: u64,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Check if account is an admin
			ensure!(<AdminAcc<T>>::contains_key(sender.clone()), Error::<T>::NotAdminAccount);

			// Ensure it is the Property Owner
			//ensure!(<PrimaryPropertyOwners<T>>::get(property_id).unwrap() == sender.clone(),<Error<T>>::NotPropertyOwner);

			let mut property =
				Self::properties(&property_id).ok_or(<Error<T>>::PropertyNotExist)?;

			property.details = detail.clone();
			<Properties<T>>::insert(&property_id, property);

			// Deposit a "PriceSet" event.
			Self::deposit_event(Event::DetailModified(sender, property_id));

			Ok(())
		}

		/// Buy a property token
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn buy_propertytoken(
			origin: OriginFor<T>,
			property_id: T::Hash,
			nooftokens: i32,
		) -> DispatchResult {
			//Get the primary owner of the property
			let primaryowner = Self::primary_property_owners(property_id).unwrap();
			//Calculate Total crypto price of transaction
			let buyer = ensure_signed(origin)?;
			let tokenprice = Self::token_price(property_id).unwrap();
			let cryptodifference = 1 / tokenprice;
			//i32 to u32
			// Because we cannot convert direct from i32 to balance
			let totalcryptoprice: u32 = (cryptodifference * nooftokens).try_into().unwrap();
			//Ensure available Tokens
			ensure!(
				Self::token_circulation(property_id).unwrap() - nooftokens >= 0,
				<Error<T>>::NoAvailableTokens
			);
			//Balance conversion
			let buy_price = Into::<BalanceOf<T>>::into(totalcryptoprice);
			// Helper to handle payment
			if (Self::finalisetransaction(
				primaryowner.clone(),
				buyer.clone(),
				totalcryptoprice,
				true,
			)) {
				//Decrease Token circulation
				let mut old_circulation = Self::token_circulation(property_id).unwrap();

				let new_circulation = old_circulation - nooftokens;

				<TokenCirculation<T>>::insert(property_id, new_circulation);
				//Converxsion
				let no_of_tokens_u32: u32 = nooftokens.try_into().unwrap();
				//Check if the buyer already owns some token for the property
				if (<SecondaryOwner<T>>::contains_key((property_id, buyer.clone()))) {
					//Update PropertiesOwnership
					//Get the index
					let index = Self::secondary_owners((property_id, buyer.clone()));

					//Get the old token balance for the buyer
					let (buyer, originaltokenbalance) =
						Self::properties_ownership((property_id, index));
					//Update the new token balance for the buyer
					let new_token_balance: u32 = originaltokenbalance + no_of_tokens_u32;
					<PropertiesOwnership<T>>::insert(
						(property_id, index),
						(buyer.clone(), new_token_balance),
					)
				} else {
					//Get SecondaryOwnerCount for a Property
					let originalcount = Self::secondary_owner_index(property_id);
					let newcount = originalcount + 1;
					<SecondaryOwnerIndex<T>>::insert(property_id, newcount);
					//Update SecondaryOwner
					<SecondaryOwner<T>>::insert((property_id, buyer.clone()), newcount);
					//Update PropertiesOwnership
					<PropertiesOwnership<T>>::insert(
						(property_id, newcount),
						(buyer.clone(), no_of_tokens_u32),
					);
				}

				//Get the token balance for the primary owner
				//Update the token balance for the primary owner

				//Get the old token balance for the buyer
				let (primaryowner, originaltokenbalance) =
					Self::properties_ownership((property_id, 0));
				//Update the new token balance for the buyer
				let new_token_balance: u32 = originaltokenbalance - no_of_tokens_u32;
				<PropertiesOwnership<T>>::insert(
					(property_id, 0),
					(primaryowner.clone(), new_token_balance),
				);

				Self::deposit_event(Event::Bought(buyer.clone(), property_id));
			}
			Ok(())
		}

		/// Sell a property token
		#[pallet::weight((0, DispatchClass::Normal, Pays::No))]
		pub fn sell_propertytoken(
			origin: OriginFor<T>,
			property_id: T::Hash,
			nooftokens: i32,
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;

			if (<SecondaryOwner<T>>::contains_key((property_id, seller.clone()))) {
				//Check if the seller already owns some token for the property
				//Update PropertiesOwnership
				//Get the index
				let index = Self::secondary_owners((property_id, seller.clone()));

				//Get the old token balance
				let (seller, originaltokenbalance) =
					Self::properties_ownership((property_id, index));
				//Check if there are sufficent tokens
				//Converxsion
				let no_of_tokens_u32: u32 = nooftokens.try_into().unwrap();
				if (originaltokenbalance >= no_of_tokens_u32) {
					//Get the primary owner of the property
					let primaryowner = Self::primary_property_owners(property_id).unwrap();
					//Calculate Total crypto price of transaction
					let tokenprice = Self::token_price(property_id).unwrap();
					let cryptodifference = 1 / tokenprice;
					//i32 to u32
					let totalcryptoprice: u32 = (cryptodifference * nooftokens).try_into().unwrap();
					// Helper to handle payment
					if (Self::finalisetransaction(
						seller.clone(),
						primaryowner.clone(),
						totalcryptoprice,
						false,
					)) {
						//Increase Token circulation
						let mut old_circulation = Self::token_circulation(property_id).unwrap();

						let new_circulation = old_circulation + nooftokens;

						<TokenCirculation<T>>::insert(property_id, new_circulation);
						//Change the token balance
						let new_token_balance: u32 = originaltokenbalance - no_of_tokens_u32;
						<PropertiesOwnership<T>>::insert(
							(property_id, index),
							(seller.clone(), new_token_balance),
						);

						//Get the token balance for the primary owner
						//Update the token balance for the primary owner

						//Get the old token balance for the buyer
						let (primaryowner, originaltokenbalance) =
							Self::properties_ownership((property_id, 0));
						//Update the new token balance for the buyer
						let new_token_balance: u32 = originaltokenbalance + no_of_tokens_u32;
						<PropertiesOwnership<T>>::insert(
							(property_id, 0),
							(primaryowner.clone(), new_token_balance),
						);
					}
				} else {
					Self::deposit_event(Event::SellError(seller.clone(), property_id));
				}

				Self::deposit_event(Event::Sold(seller.clone(), property_id));
			}
			Ok(())
		}
	}

	//** Our helper functions.**//

	impl<T: Config> Pallet<T> {
		// Helper to mint a Property.
		pub fn mint(owner: &T::AccountId, details: u64, price: i32) -> Result<T::Hash, Error<T>> {
			let property = Property { details, initialprice: price };
			// Calculate the Property Id
			let property_id = T::Hashing::hash_of(&property);

			// Increment the Property Count
			let new_cnt = Self::property_cnt().checked_add(1).unwrap();
			<PropertyCnt<T>>::put(new_cnt);
			// Insert the Property data
			<Properties<T>>::insert(property_id, property);
			// Set the property price
			<PropertyPrice<T>>::insert(property_id, price);
			// Insert the primary property owner
			<PrimaryPropertyOwners<T>>::insert(property_id, owner);

			// Update the token TokenCirculation
			// We create tokens equivalent to the price of the property
			<TokenCirculation<T>>::insert(property_id, price);

			// We initalise the token price as 1 token = 1 crypto currency
			// Need to update when price of property increases or decreases
			<TokenPrice<T>>::insert(property_id, 1);

			// Add Tokens for the Owner Account at index 0 of SecondaryPropertiesOwnership
			//property id, index 0, property owner, initialise the tokens balance
			//Conversion
			let price_u32: u32 = price.try_into().unwrap();
			<PropertiesOwnership<T>>::insert((property_id, 0), (owner, price_u32));

			Ok(property_id)
		}

		// Helper to burn a Property.
		pub fn burn(property_id: T::Hash) -> bool {
			let ownercount = Self::secondary_owner_index(property_id);
			// Check if the token balance of secondary owners is zero
			for i in 0..ownercount {
				let (secowner, tokenbalance) = Self::properties_ownership((property_id, i));
				if (tokenbalance == 0) {
					<PropertiesOwnership<T>>::remove((property_id, i));
					<SecondaryOwner<T>>::remove((property_id, secowner.clone()));
				} else {
					return false;
				}
			}
			<SecondaryOwnerIndex<T>>::remove(property_id);
			// Derement the Property Count
			let new_cnt = Self::property_cnt().checked_sub(1).unwrap();
			<PropertyCnt<T>>::put(new_cnt);
			// Remove the Property data
			<Properties<T>>::remove(property_id);
			// Remove the property price
			<PropertyPrice<T>>::remove(property_id);
			// Remove the primary property owner
			<PrimaryPropertyOwners<T>>::remove(property_id);

			// Update the token TokenCirculation
			<TokenCirculation<T>>::remove(property_id);

			// We initalise the token price as 1 token = 1 crypto currency
			// Need to update when price of property increases or decreases
			<TokenPrice<T>>::remove(property_id);

			return true;
		}

		// Helper to finalise transaction payment.
		pub fn finalisetransaction(
			seller: T::AccountId,
			buyer: T::AccountId,
			transactionAmt: u32,
			isBuyToken: bool,
		) -> bool {
			//Getcomission account
			let comissionaccount = Self::comissionacc(0);
			let mut comissionpercent = 0;
			// Get comission percentage
			if (isBuyToken) {
				comissionpercent = Self::buycomission(0);
			} else {
				comissionpercent = Self::sellcomission(0);
			}
			//Calculate comisssion if buy or sell from transation Amount
			let comissionamt = (transactionAmt * comissionpercent) / 100;
			// Add the comission amt + total transaction amt
			let totalamt = transactionAmt + comissionamt;
			let totalamtcrypto = Into::<BalanceOf<T>>::into(totalamt);
			let comissioncrypto = Into::<BalanceOf<T>>::into(comissionamt);
			let transactionamtcrypto = Into::<BalanceOf<T>>::into(transactionAmt);
			//Ensure Balance for each transaction party
			info!("total{}",totalamt);
			info!("comission{}",comissionamt);
			info!("txamt{}",transactionAmt);
			//Balance check for buyer
			if (T::Currency::free_balance(&buyer) < totalamtcrypto) {
				return false;
			}
			//Balance check for seller
			if (T::Currency::free_balance(&seller) < comissioncrypto) {
				return false;
			}

			// Transfer the amount from buyer to seller
			T::Currency::transfer(
				&buyer,
				&seller,
				transactionamtcrypto,
				ExistenceRequirement::KeepAlive,
			);
			// Transfer the comission from buyer
			T::Currency::transfer(
				&buyer,
				&comissionaccount,
				comissioncrypto,
				ExistenceRequirement::KeepAlive,
			);
			// Transfer the comission from seller
			T::Currency::transfer(
				&seller,
				&comissionaccount,
				comissioncrypto,
				ExistenceRequirement::KeepAlive,
			);
			return true;
		}
	}
}
