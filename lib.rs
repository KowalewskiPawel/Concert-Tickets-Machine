#![cfg_attr(not(feature = "std"), no_std)]

#[ink::contract]
mod tickets {

    use ink::storage::Mapping;
    use ink::prelude::string::String;
    use ink::prelude::vec::Vec;
    use ink::prelude::string::ToString;

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Concert {
        concert_id: u32,
        ticket_price: u128,
        date: Timestamp,
        tickets_left: u32,
    }

    #[ink(storage)]
    pub struct Tickets {
        name: String,
        description: String,
        concert_counter: u32,
        concerts: Mapping<u32, Concert>,
        account_tickets: Mapping<AccountId, Vec<String>>,
        tickets_map: Mapping<String, AccountId>,
        tickets_owners_map: Mapping<String, String>,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum CustomError {
        ConcertDoesntExist,
        TicketsSoldOut,
        ConcertFinished,
        IncorrectPaymentValue,
        AccountNotFound,
    }

    pub type Result<T> = core::result::Result<T, CustomError>;

    impl Tickets {
        #[ink(constructor)]
        pub fn new(init_name: String, init_description: String) -> Self {
            Self {
                name: init_name,
                description: init_description,
                concert_counter: 0,
                concerts: Mapping::new(),
                account_tickets: Mapping::new(),
                tickets_map: Mapping::new(),
                tickets_owners_map: Mapping::new(),
            }
        }

        #[ink(constructor)]
        pub fn default() -> Self {
            Self {
                name: "".to_string(),
                description: "".to_string(),
                concert_counter: 0,
                concerts: Mapping::new(),
                account_tickets: Mapping::new(),
                tickets_map: Mapping::new(),
                tickets_owners_map: Mapping::new(),
            }
        }

        #[ink(message)]
        pub fn add_new_concert(&mut self, tickets_available: u32, price: u128, timestamp: Timestamp) {
            let new_concert = Concert {
                concert_id: self.concert_counter,
                ticket_price: price,
                date: timestamp,
                tickets_left: tickets_available
            };
            self.concerts.insert(self.concert_counter, &new_concert);
            self.concert_counter += 1;
        }

        #[ink(message)]
        pub fn get_concerts(&self) -> Result<Vec<Concert>> {
            let mut concerts: Vec<Concert> = Vec::new();
            for n in 0..self.concert_counter {
                let concert = self.concerts.get(n).ok_or(CustomError::ConcertDoesntExist).unwrap();
                concerts.push(concert);
            }
            Ok(concerts)
        }

        #[ink(message, payable)]
        pub fn buy_ticket(&mut self, concert_id: u32, name: String, surname: String) -> Result<()> {
                let mut concert = self.concerts.get(concert_id).ok_or(CustomError::ConcertDoesntExist).unwrap();
                if concert.tickets_left < 1 {
                    return Err(CustomError::TicketsSoldOut);
                }
                if self.env().block_timestamp() > concert.date {
                    return Err(CustomError::ConcertFinished);
                }
                let transferred_value = self.env().transferred_value();
                if transferred_value != concert.ticket_price {
                    return Err(CustomError::IncorrectPaymentValue);
                }
                let caller = self.env().caller();
                let ticket_id = String::from(concert_id.to_string() + ", " + &concert.tickets_left.to_string());
                let ticket_owner = String::from(name + " " + &surname);
                let mut owner_tickets = match self.account_tickets.get(&caller) {
                    Some(tickets) => tickets,
                    None => Vec::new()
                };
                concert.tickets_left -= 1;
                self.concerts.insert(concert_id, &concert);
                self.tickets_map.insert(&ticket_id, &caller);
                self.tickets_owners_map.insert(&ticket_id, &ticket_owner);
                owner_tickets.push(ticket_id);
                self.account_tickets.insert(&caller, &owner_tickets);
                Ok(())
        }

        #[ink(message)]
        pub fn get_my_tickets(&self) -> Result<Vec<String>> {
            let caller = self.env().caller();
            match self.account_tickets.get(caller) {
                Some(tickets) => return Ok(tickets),
                None => return Err(CustomError::AccountNotFound)
            };
        }
    }

    #[cfg(test)]
    mod tests {
        use ink::env::pay_with_call;

        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let mut tickets = Tickets::default();
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let contract = ink::env::account_id::<ink::env::DefaultEnvironment>();
            ink::env::test::set_callee::<ink::env::DefaultEnvironment>(contract);
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(accounts.bob, 100);
            assert_eq!(tickets.add_new_concert(30, 30, 1780600226001), ());
            assert_eq!(pay_with_call!(tickets.buy_ticket(0, "pawel".to_string(), "doe".to_string()), 30) , Ok(()));
            assert_eq!(tickets.get_my_tickets(), Ok(vec!["0, 30".to_string()]));
        }
    }
}
