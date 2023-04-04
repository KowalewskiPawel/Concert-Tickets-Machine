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
        ticket_price: u32,
        date: Timestamp,
        tickets_left: u32,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct AccountTickets {
        tickets: Vec<String>
    }

    #[ink(storage)]
    pub struct Tickets {
        name: String,
        description: String,
        concert_counter: u32,
        concerts: Mapping<u32, Concert>,
        account_tickets: Mapping<AccountId, AccountTickets>,
        tickets_map: Mapping<String, AccountId>,
        tickets_owners_map: Mapping<String, String>,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum CustomError {
        ConcertDoesntExist,
        TicketsSoldOut,
        ConcertFinished,
        AccountNotFound
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
        pub fn add_new_concert(&mut self, tickets_available: u32, price: u32, timestamp: Timestamp) {
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
        pub fn get(&self) -> Result<Vec<Concert>> {
            let mut concerts: Vec<Concert> = Vec::new();
            for n in 0..self.concert_counter {
                let concert = self.concerts.get(n).ok_or(CustomError::ConcertDoesntExist).unwrap();
                concerts.push(concert);
            }
            Ok(concerts)
        }

        #[ink(message)]
        pub fn buy_tikcet(&mut self, concert_id: u32, name: String, surname: String) -> Result<()> {
                let mut concert = self.concerts.get(concert_id).ok_or(CustomError::ConcertDoesntExist).unwrap();
                if concert.tickets_left < 1 {
                    return Err(CustomError::TicketsSoldOut);
                }
                if self.env().block_timestamp() > concert.date {
                    return Err(CustomError::ConcertFinished);
                }
                let caller = self.env().caller();
                let ticket_id = String::from(concert_id.to_string() + ", " + &concert.tickets_left.to_string());
                let ticket_owner = String::from(name + " " + &surname);
                let mut owner_tickets = self.account_tickets.get(&caller).unwrap();
                concert.tickets_left -= 1;
                self.concerts.insert(concert_id, &concert);
                self.tickets_map.insert(&ticket_id, &caller);
                self.tickets_owners_map.insert(&ticket_id, &ticket_owner);
                owner_tickets.tickets.push(ticket_id);
                self.account_tickets.insert(&caller, &owner_tickets);
                Ok(())
        }

        #[ink(message)]
        pub fn get_my_tickets(&self) -> Result<Vec<String>> {
            let caller = self.env().caller();
            Ok(self.account_tickets.get(caller).unwrap().tickets)
        }
    }

    // #[cfg(test)]
    // mod tests {
    //     /// Imports all the definitions from the outer scope so we can use them here.
    //     use super::*;

    //     /// We test if the default constructor does its job.
    //     #[ink::test]
    //     fn default_works() {
    //         let tickets = Tickets::default();
    //         assert_eq!(tickets.get(), false);
    //     }

    //     /// We test a simple use case of our contract.
    //     #[ink::test]
    //     fn it_works() {
    //         let mut tickets = Tickets::new(false);
    //         assert_eq!(tickets.get(), false);
    //         tickets.flip();
    //         assert_eq!(tickets.get(), true);
    //     }
    // }
}
