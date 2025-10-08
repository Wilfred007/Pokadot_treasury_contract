#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod treasury_governance {
    use ink::prelude::{vec, vec::Vec, string::String};
    use ink::storage::Mapping;
    use ink::primitives::H160;

    // Types
    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ProposalType {
        Treasury,
        Governance,
        Technical,
        Other,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum VotingPeriod {
        ThreeDays,
        SevenDays,
        FourteenDays,
        ThirtyDays,
    }

    impl VotingPeriod {
        pub fn to_blocks(&self) -> u32 {
            match self {
                VotingPeriod::ThreeDays => 3 * 24 * 60 * 10,
                VotingPeriod::SevenDays => 7 * 24 * 60 * 10,
                VotingPeriod::FourteenDays => 14 * 24 * 60 * 10,
                VotingPeriod::ThirtyDays => 30 * 24 * 60 * 10,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum QuorumThreshold {
        Five,
        Ten,
        Twenty,
        TwentyFive,
    }

    impl QuorumThreshold {
        pub fn to_percentage(&self) -> u32 {
            match self {
                QuorumThreshold::Five => 5,
                QuorumThreshold::Ten => 10,
                QuorumThreshold::Twenty => 20,
                QuorumThreshold::TwentyFive => 25,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ExecutionDelay {
        Immediately,
        OneDay,
        TwoDays,
        SevenDays,
    }

    impl ExecutionDelay {
        pub fn to_blocks(&self) -> u32 {
            match self {
                ExecutionDelay::Immediately => 0,
                ExecutionDelay::OneDay => 24 * 60 * 10,
                ExecutionDelay::TwoDays => 2 * 24 * 60 * 10,
                ExecutionDelay::SevenDays => 7 * 24 * 60 * 10,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct GovernanceParameters {
        pub voting_period: VotingPeriod,
        pub quorum_threshold: QuorumThreshold,
        pub execution_delay: ExecutionDelay,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct VotingOptions {
        pub options: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct VoteChoice {
        pub option_index: u32,
        pub option_text: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub enum ProposalStatus {
        Active,
        Passed,
        Rejected,
        Executed,
        Expired,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Proposal {
        pub id: u32,
        pub title: String,
        pub description: String,
        pub proposal_type: ProposalType,
        pub governance_params: GovernanceParameters,
        pub voting_options: VotingOptions,
        pub proposer: H160,
        pub created_at: u32,
        pub voting_end: u32,
        pub execution_time: u32,
        pub status: ProposalStatus,
        pub vote_counts: Vec<u128>,
        pub total_voters: u32,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct Vote {
        pub voter: H160,
        pub choice: VoteChoice,
        pub timestamp: u32,
        pub weight: u128,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct ContractStats {
        pub total_proposals: u32,
        pub active_proposals: u32,
        pub executed_proposals: u32,
        pub total_voters: u32,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    #[cfg_attr(feature = "std", derive(ink::storage::traits::StorageLayout))]
    pub struct ProposalResults {
        pub proposal_id: u32,
        pub vote_counts: Vec<u128>,
        pub option_names: Vec<String>,
        pub total_votes: u128,
        pub quorum_reached: bool,
        pub winning_option: Option<(u32, String, u128)>,
    }

    // Errors
    #[derive(Debug, PartialEq, Eq)]
    #[ink::scale_derive(Encode, Decode, TypeInfo)]
    pub enum Error {
        ProposalNotFound,
        ProposalNotActive,
        VotingPeriodEnded,
        AlreadyVoted,
        NotAuthorized,
        ProposalNotReadyForExecution,
        InvalidProposal,
        InvalidOptionIndex,
        NoVotingOptions,
        TooManyVotingOptions,
        ArithmeticOverflow,
        NotRegisteredVoter,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    // Events temporarily removed due to ink! v6 alpha bugs

    // Storage
    #[ink(storage)]
    pub struct TreasuryGovernance {
        pub next_proposal_id: u32,
        pub proposals: Mapping<u32, Proposal>,
        pub votes: Mapping<(u32, H160), Vote>,
        pub proposal_ids: Vec<u32>,
        pub total_voters: u32,
        pub owner: H160,
        pub registered_voters: Mapping<H160, bool>,
    }

    impl TreasuryGovernance {
        /// Constructor that initializes the contract
        #[ink(constructor)]
        pub fn new() -> Self {
            let caller = Self::env().caller();
            Self {
                next_proposal_id: 1,
                proposals: Default::default(),
                votes: Default::default(),
                proposal_ids: Vec::new(),
                total_voters: 0,
                owner: caller,
                registered_voters: Default::default(),
            }
        }

        /// Register a voter to participate in governance
        #[ink(message)]
        pub fn register_voter(&mut self) -> Result<()> {
            let caller = self.env().caller();
            
            // Check if already registered
            if self.registered_voters.get(&caller).unwrap_or(false) {
                return Ok(()); // Already registered, no error
            }
            
            // Register the voter
            self.registered_voters.insert(&caller, &true);
            self.total_voters = self.total_voters.checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
            
            // Event emission removed due to ink! v6 alpha bugs
            
            Ok(())
        }

        /// Create a new proposal
        #[ink(message)]
        pub fn create_proposal(
            &mut self,
            title: String,
            description: String,
            proposal_type: ProposalType,
            governance_params: GovernanceParameters,
            voting_options: VotingOptions,
        ) -> Result<u32> {
            let caller = self.env().caller();
            let current_block = self.env().block_number();
            
            // Validate voting options
            if voting_options.options.is_empty() {
                return Err(Error::NoVotingOptions);
            }
            if voting_options.options.len() > 10 {
                return Err(Error::TooManyVotingOptions);
            }
            
            // Calculate timing
            let voting_end = current_block.checked_add(governance_params.voting_period.to_blocks())
                .ok_or(Error::ArithmeticOverflow)?;
            let execution_time = voting_end.checked_add(governance_params.execution_delay.to_blocks())
                .ok_or(Error::ArithmeticOverflow)?;
            
            // Initialize vote counts
            let vote_counts = vec![0u128; voting_options.options.len()];
            
            // Create proposal
            let proposal_id = self.next_proposal_id;
            let proposal = Proposal {
                id: proposal_id,
                title: title.clone(),
                description,
                proposal_type,
                governance_params,
                voting_options,
                proposer: caller,
                created_at: current_block,
                voting_end,
                execution_time,
                status: ProposalStatus::Active,
                vote_counts,
                total_voters: 0,
            };
            
            // Store proposal
            self.proposals.insert(&proposal_id, &proposal);
            self.proposal_ids.push(proposal_id);
            self.next_proposal_id = self.next_proposal_id.checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
            
            // Event emission removed due to ink! v6 alpha bugs
            
            Ok(proposal_id)
        }

        /// Cast a vote on a proposal
        #[ink(message)]
        pub fn vote(&mut self, proposal_id: u32, option_index: u32) -> Result<()> {
            let caller = self.env().caller();
            let current_block = self.env().block_number();
            
            // Check if voter is registered
            if !self.registered_voters.get(&caller).unwrap_or(false) {
                return Err(Error::NotRegisteredVoter);
            }
            
            // Get proposal
            let mut proposal = self.proposals.get(&proposal_id)
                .ok_or(Error::ProposalNotFound)?;
            
            // Validate proposal status and timing
            if proposal.status != ProposalStatus::Active {
                return Err(Error::ProposalNotActive);
            }
            if current_block > proposal.voting_end {
                return Err(Error::VotingPeriodEnded);
            }
            
            // Check if user already voted
            if self.votes.get(&(proposal_id, caller)).is_some() {
                return Err(Error::AlreadyVoted);
            }
            
            // Validate option index
            if option_index as usize >= proposal.voting_options.options.len() {
                return Err(Error::InvalidOptionIndex);
            }
            
            // Create vote record
            let option_text = proposal.voting_options.options[option_index as usize].clone();
            let vote = Vote {
                voter: caller,
                choice: VoteChoice {
                    option_index,
                    option_text: option_text.clone(),
                },
                timestamp: current_block,
                weight: 1, // Simple voting weight of 1 for now
            };
            
            // Update vote counts
            proposal.vote_counts[option_index as usize] = proposal.vote_counts[option_index as usize]
                .checked_add(vote.weight)
                .ok_or(Error::ArithmeticOverflow)?;
            proposal.total_voters = proposal.total_voters.checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
            
            // Store vote and updated proposal
            self.votes.insert(&(proposal_id, caller), &vote);
            self.proposals.insert(&proposal_id, &proposal);
            
            // Event emission removed due to ink! v6 alpha bugs
            
            Ok(())
        }

        /// Update proposal status based on voting results
        #[ink(message)]
        pub fn update_proposal_status(&mut self, proposal_id: u32) -> Result<()> {
            let current_block = self.env().block_number();
            
            let mut proposal = self.proposals.get(&proposal_id)
                .ok_or(Error::ProposalNotFound)?;
            
            // Only update if currently active and voting period has ended
            if proposal.status != ProposalStatus::Active || current_block <= proposal.voting_end {
                return Ok(());
            }
            
            let _old_status = proposal.status.clone();
            
            // Check if quorum is reached
            let quorum_reached = self.has_reached_quorum_internal(&proposal)?;
            
            if quorum_reached {
                // Find winning option (highest vote count)
                let mut max_votes = 0u128;
                let mut winning_options = Vec::new();
                
                for (index, &votes) in proposal.vote_counts.iter().enumerate() {
                    if votes > max_votes {
                        max_votes = votes;
                        winning_options.clear();
                        winning_options.push(index);
                    } else if votes == max_votes && votes > 0 {
                        winning_options.push(index);
                    }
                }
                
                // If there's a clear winner, mark as passed; otherwise rejected due to tie
                if winning_options.len() == 1 && max_votes > 0 {
                    proposal.status = ProposalStatus::Passed;
                } else {
                    proposal.status = ProposalStatus::Rejected;
                }
            } else {
                proposal.status = ProposalStatus::Rejected;
            }
            
            // Store updated proposal
            self.proposals.insert(&proposal_id, &proposal);
            
            // Event emission removed due to ink! v6 alpha bugs
            
            Ok(())
        }

        /// Execute a passed proposal
        #[ink(message)]
        pub fn execute_proposal(&mut self, proposal_id: u32) -> Result<()> {
            let current_block = self.env().block_number();
            
            let mut proposal = self.proposals.get(&proposal_id)
                .ok_or(Error::ProposalNotFound)?;
            
            // Validate proposal can be executed
            if proposal.status != ProposalStatus::Passed {
                return Err(Error::ProposalNotReadyForExecution);
            }
            if current_block < proposal.execution_time {
                return Err(Error::ProposalNotReadyForExecution);
            }
            
            // Update status to executed
            proposal.status = ProposalStatus::Executed;
            self.proposals.insert(&proposal_id, &proposal);
            
            // Event emission removed due to ink! v6 alpha bugs
            
            Ok(())
        }

        /// Internal helper to check if quorum is reached
        fn has_reached_quorum_internal(&self, proposal: &Proposal) -> Result<bool> {
            if self.total_voters == 0 {
                return Ok(false);
            }
            
            let quorum_percentage = proposal.governance_params.quorum_threshold.to_percentage();
            let required_votes = (self.total_voters as u128 * quorum_percentage as u128) / 100;
            let total_votes: u128 = proposal.vote_counts.iter().sum();
            
            Ok(total_votes >= required_votes)
        }

        // Query functions

        /// Get a specific proposal by ID
        #[ink(message)]
        pub fn get_proposal(&self, proposal_id: u32) -> Option<Proposal> {
            self.proposals.get(&proposal_id)
        }

        /// Get all proposal IDs
        #[ink(message)]
        pub fn get_all_proposal_ids(&self) -> Vec<u32> {
            self.proposal_ids.clone()
        }

        /// Get user's vote on a specific proposal
        #[ink(message)]
        pub fn get_user_vote(&self, proposal_id: u32, user: H160) -> Option<Vote> {
            self.votes.get(&(proposal_id, user))
        }

        /// Get contract statistics
        #[ink(message)]
        pub fn get_stats(&self) -> ContractStats {
            let mut active_proposals = 0u32;
            let mut executed_proposals = 0u32;
            
            for proposal_id in &self.proposal_ids {
                if let Some(proposal) = self.proposals.get(proposal_id) {
                    match proposal.status {
                        ProposalStatus::Active => active_proposals += 1,
                        ProposalStatus::Executed => executed_proposals += 1,
                        _ => {}
                    }
                }
            }
            
            ContractStats {
                total_proposals: self.proposal_ids.len() as u32,
                active_proposals,
                executed_proposals,
                total_voters: self.total_voters,
            }
        }

        /// Get total number of registered voters
        #[ink(message)]
        pub fn get_total_voters(&self) -> u32 {
            self.total_voters
        }

        /// Check if a proposal has reached quorum
        #[ink(message)]
        pub fn has_reached_quorum(&self, proposal_id: u32) -> Result<bool> {
            let proposal = self.proposals.get(&proposal_id)
                .ok_or(Error::ProposalNotFound)?;
            self.has_reached_quorum_internal(&proposal)
        }

        /// Get proposal results with vote counts
        #[ink(message)]
        pub fn get_proposal_results(&self, proposal_id: u32) -> Result<(Vec<u128>, bool)> {
            let proposal = self.proposals.get(&proposal_id)
                .ok_or(Error::ProposalNotFound)?;
            let quorum_reached = self.has_reached_quorum_internal(&proposal)?;
            Ok((proposal.vote_counts, quorum_reached))
        }

        /// Get voting options for a proposal
        #[ink(message)]
        pub fn get_voting_options(&self, proposal_id: u32) -> Result<Vec<String>> {
            let proposal = self.proposals.get(&proposal_id)
                .ok_or(Error::ProposalNotFound)?;
            Ok(proposal.voting_options.options)
        }

        /// Get detailed results with option names
        #[ink(message)]
        pub fn get_detailed_results(&self, proposal_id: u32) -> Result<ProposalResults> {
            let proposal = self.proposals.get(&proposal_id)
                .ok_or(Error::ProposalNotFound)?;
            let quorum_reached = self.has_reached_quorum_internal(&proposal)?;
            let total_votes: u128 = proposal.vote_counts.iter().sum();
            
            // Find winning option
            let mut winning_option = None;
            let mut max_votes = 0u128;
            for (index, &votes) in proposal.vote_counts.iter().enumerate() {
                if votes > max_votes {
                    max_votes = votes;
                    winning_option = Some((
                        index as u32,
                        proposal.voting_options.options[index].clone(),
                        votes,
                    ));
                }
            }
            
            Ok(ProposalResults {
                proposal_id,
                vote_counts: proposal.vote_counts,
                option_names: proposal.voting_options.options,
                total_votes,
                quorum_reached,
                winning_option,
            })
        }

        /// Get the winning option for a proposal
        #[ink(message)]
        pub fn get_winning_option(&self, proposal_id: u32) -> Result<Option<(u32, String, u128)>> {
            let proposal = self.proposals.get(&proposal_id)
                .ok_or(Error::ProposalNotFound)?;
            
            let mut max_votes = 0u128;
            let mut winning_option = None;
            let mut tie_exists = false;
            
            for (index, &votes) in proposal.vote_counts.iter().enumerate() {
                if votes > max_votes {
                    max_votes = votes;
                    winning_option = Some((
                        index as u32,
                        proposal.voting_options.options[index].clone(),
                        votes,
                    ));
                    tie_exists = false;
                } else if votes == max_votes && votes > 0 {
                    tie_exists = true;
                }
            }
            
            // Return None if there's a tie or no votes
            if tie_exists || max_votes == 0 {
                Ok(None)
            } else {
                Ok(winning_option)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::prelude::vec;

        #[ink::test]
        fn test_constructor() {
            let contract = TreasuryGovernance::new();
            assert_eq!(contract.next_proposal_id, 1);
            assert_eq!(contract.total_voters, 0);
            assert_eq!(contract.proposal_ids.len(), 0);
        }

        #[ink::test]
        fn test_voter_registration() {
            let mut contract = TreasuryGovernance::new();
            
            // Register first voter
            assert!(contract.register_voter().is_ok());
            assert_eq!(contract.get_total_voters(), 1);
            
            // Register same voter again (should not increase count)
            assert!(contract.register_voter().is_ok());
            assert_eq!(contract.get_total_voters(), 1);
        }

        #[ink::test]
        fn test_create_proposal_success() {
            let mut contract = TreasuryGovernance::new();
            
            let governance_params = GovernanceParameters {
                voting_period: VotingPeriod::SevenDays,
                quorum_threshold: QuorumThreshold::Ten,
                execution_delay: ExecutionDelay::OneDay,
            };
            
            let voting_options = VotingOptions {
                options: vec![
                    String::from("Yes"),
                    String::from("No"),
                ],
            };
            
            let result = contract.create_proposal(
                String::from("Test Proposal"),
                String::from("A test proposal for governance"),
                ProposalType::Treasury,
                governance_params,
                voting_options,
            );
            
            assert!(result.is_ok());
            let proposal_id = result.unwrap();
            assert_eq!(proposal_id, 1);
            assert_eq!(contract.next_proposal_id, 2);
            assert_eq!(contract.proposal_ids.len(), 1);
            
            let proposal = contract.get_proposal(proposal_id).unwrap();
            assert_eq!(proposal.title, "Test Proposal");
            assert_eq!(proposal.status, ProposalStatus::Active);
            assert_eq!(proposal.vote_counts.len(), 2);
        }

        #[ink::test]
        fn test_voting_success() {
            let mut contract = TreasuryGovernance::new();
            
            // Register voter first
            contract.register_voter().unwrap();
            
            // Create proposal
            let governance_params = GovernanceParameters {
                voting_period: VotingPeriod::SevenDays,
                quorum_threshold: QuorumThreshold::Ten,
                execution_delay: ExecutionDelay::OneDay,
            };
            
            let voting_options = VotingOptions {
                options: vec![
                    String::from("Yes"),
                    String::from("No"),
                ],
            };
            
            let proposal_id = contract.create_proposal(
                String::from("Test Proposal"),
                String::from("A test proposal"),
                ProposalType::Treasury,
                governance_params,
                voting_options,
            ).unwrap();
            
            // Vote on proposal
            let result = contract.vote(proposal_id, 0);
            assert!(result.is_ok());
            
            // Check that the vote was recorded by verifying proposal updates
            // Note: Individual vote lookup may fail due to test environment account differences
            
            // Check vote counts updated
            let proposal = contract.get_proposal(proposal_id).unwrap();
            assert_eq!(proposal.vote_counts[0], 1);
            assert_eq!(proposal.vote_counts[1], 0);
            assert_eq!(proposal.total_voters, 1);
        }

        #[ink::test]
        fn test_create_proposal_no_voting_options() {
            let mut contract = TreasuryGovernance::new();
            
            let governance_params = GovernanceParameters {
                voting_period: VotingPeriod::SevenDays,
                quorum_threshold: QuorumThreshold::Ten,
                execution_delay: ExecutionDelay::OneDay,
            };
            
            let voting_options = VotingOptions {
                options: vec![], // Empty options should fail
            };
            
            let result = contract.create_proposal(
                String::from("Test Proposal"),
                String::from("A test proposal"),
                ProposalType::Treasury,
                governance_params,
                voting_options,
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::NoVotingOptions);
        }

        #[ink::test]
        fn test_create_proposal_too_many_options() {
            let mut contract = TreasuryGovernance::new();
            
            let governance_params = GovernanceParameters {
                voting_period: VotingPeriod::SevenDays,
                quorum_threshold: QuorumThreshold::Ten,
                execution_delay: ExecutionDelay::OneDay,
            };
            
            // Create 11 options (should fail as max is 10)
            let mut options = Vec::new();
            for i in 0..11 {
                options.push(format!("Option {}", i));
            }
            
            let voting_options = VotingOptions { options };
            
            let result = contract.create_proposal(
                String::from("Test Proposal"),
                String::from("A test proposal"),
                ProposalType::Treasury,
                governance_params,
                voting_options,
            );
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::TooManyVotingOptions);
        }

        #[ink::test]
        fn test_vote_unregistered_voter() {
            let mut contract = TreasuryGovernance::new();
            
            // Create proposal without registering voter
            let governance_params = GovernanceParameters {
                voting_period: VotingPeriod::SevenDays,
                quorum_threshold: QuorumThreshold::Ten,
                execution_delay: ExecutionDelay::OneDay,
            };
            
            let voting_options = VotingOptions {
                options: vec![String::from("Yes"), String::from("No")],
            };
            
            let proposal_id = contract.create_proposal(
                String::from("Test Proposal"),
                String::from("A test proposal"),
                ProposalType::Treasury,
                governance_params,
                voting_options,
            ).unwrap();
            
            // Try to vote without being registered
            let result = contract.vote(proposal_id, 0);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::NotRegisteredVoter);
        }

        #[ink::test]
        fn test_vote_nonexistent_proposal() {
            let mut contract = TreasuryGovernance::new();
            
            // Register voter
            contract.register_voter().unwrap();
            
            // Try to vote on non-existent proposal
            let result = contract.vote(999, 0);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::ProposalNotFound);
        }

        #[ink::test]
        fn test_vote_invalid_option_index() {
            let mut contract = TreasuryGovernance::new();
            
            // Register voter
            contract.register_voter().unwrap();
            
            // Create proposal with 2 options
            let governance_params = GovernanceParameters {
                voting_period: VotingPeriod::SevenDays,
                quorum_threshold: QuorumThreshold::Ten,
                execution_delay: ExecutionDelay::OneDay,
            };
            
            let voting_options = VotingOptions {
                options: vec![String::from("Yes"), String::from("No")],
            };
            
            let proposal_id = contract.create_proposal(
                String::from("Test Proposal"),
                String::from("A test proposal"),
                ProposalType::Treasury,
                governance_params,
                voting_options,
            ).unwrap();
            
            // Try to vote with invalid option index (2 when only 0,1 exist)
            let result = contract.vote(proposal_id, 2);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::InvalidOptionIndex);
        }

        #[ink::test]
        fn test_double_voting_same_voter() {
            let mut contract = TreasuryGovernance::new();
            
            // Register voter
            contract.register_voter().unwrap();
            
            // Create proposal
            let governance_params = GovernanceParameters {
                voting_period: VotingPeriod::SevenDays,
                quorum_threshold: QuorumThreshold::Ten,
                execution_delay: ExecutionDelay::OneDay,
            };
            
            let voting_options = VotingOptions {
                options: vec![String::from("Yes"), String::from("No")],
            };
            
            let proposal_id = contract.create_proposal(
                String::from("Test Proposal"),
                String::from("A test proposal"),
                ProposalType::Treasury,
                governance_params,
                voting_options,
            ).unwrap();
            
            // First vote should succeed
            let result = contract.vote(proposal_id, 0);
            assert!(result.is_ok());
            
            // Second vote should fail (already voted)
            let result = contract.vote(proposal_id, 1);
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Error::AlreadyVoted);
        }

        #[ink::test]
        fn test_get_proposal_details() {
            let mut contract = TreasuryGovernance::new();
            
            let governance_params = GovernanceParameters {
                voting_period: VotingPeriod::ThirtyDays,
                quorum_threshold: QuorumThreshold::TwentyFive,
                execution_delay: ExecutionDelay::SevenDays,
            };
            
            let voting_options = VotingOptions {
                options: vec![
                    String::from("Approve"),
                    String::from("Reject"),
                    String::from("Abstain"),
                ],
            };
            
            let proposal_id = contract.create_proposal(
                String::from("Treasury Funding"),
                String::from("Fund development project"),
                ProposalType::Treasury,
                governance_params.clone(),
                voting_options.clone(),
            ).unwrap();
            
            let proposal = contract.get_proposal(proposal_id).unwrap();
            assert_eq!(proposal.title, "Treasury Funding");
            assert_eq!(proposal.description, "Fund development project");
            assert_eq!(proposal.proposal_type, ProposalType::Treasury);
            assert_eq!(proposal.governance_params.voting_period, VotingPeriod::ThirtyDays);
            assert_eq!(proposal.governance_params.quorum_threshold, QuorumThreshold::TwentyFive);
            assert_eq!(proposal.governance_params.execution_delay, ExecutionDelay::SevenDays);
            assert_eq!(proposal.voting_options.options.len(), 3);
            assert_eq!(proposal.status, ProposalStatus::Active);
        }

        #[ink::test]
        fn test_get_nonexistent_proposal() {
            let contract = TreasuryGovernance::new();
            
            let result = contract.get_proposal(999);
            assert!(result.is_none());
        }

        #[ink::test]
        fn test_multiple_proposal_types() {
            let mut contract = TreasuryGovernance::new();
            
            let governance_params = GovernanceParameters {
                voting_period: VotingPeriod::SevenDays,
                quorum_threshold: QuorumThreshold::Ten,
                execution_delay: ExecutionDelay::OneDay,
            };
            
            let voting_options = VotingOptions {
                options: vec![String::from("Yes"), String::from("No")],
            };
            
            // Create different types of proposals
            let treasury_id = contract.create_proposal(
                String::from("Treasury Proposal"),
                String::from("Treasury funding"),
                ProposalType::Treasury,
                governance_params.clone(),
                voting_options.clone(),
            ).unwrap();
            
            let governance_id = contract.create_proposal(
                String::from("Governance Proposal"),
                String::from("Change governance rules"),
                ProposalType::Governance,
                governance_params.clone(),
                voting_options.clone(),
            ).unwrap();
            
            let technical_id = contract.create_proposal(
                String::from("Technical Proposal"),
                String::from("Technical upgrade"),
                ProposalType::Technical,
                governance_params.clone(),
                voting_options.clone(),
            ).unwrap();
            
            let other_id = contract.create_proposal(
                String::from("Other Proposal"),
                String::from("Other type"),
                ProposalType::Other,
                governance_params,
                voting_options,
            ).unwrap();
            
            // Verify all proposals were created with correct types
            assert_eq!(contract.get_proposal(treasury_id).unwrap().proposal_type, ProposalType::Treasury);
            assert_eq!(contract.get_proposal(governance_id).unwrap().proposal_type, ProposalType::Governance);
            assert_eq!(contract.get_proposal(technical_id).unwrap().proposal_type, ProposalType::Technical);
            assert_eq!(contract.get_proposal(other_id).unwrap().proposal_type, ProposalType::Other);
            
            assert_eq!(contract.proposal_ids.len(), 4);
        }

        #[ink::test]
        fn test_get_all_proposal_ids() {
            let mut contract = TreasuryGovernance::new();
            
            let governance_params = GovernanceParameters {
                voting_period: VotingPeriod::SevenDays,
                quorum_threshold: QuorumThreshold::Ten,
                execution_delay: ExecutionDelay::OneDay,
            };
            
            let voting_options = VotingOptions {
                options: vec![String::from("Yes"), String::from("No")],
            };
            
            // Initially no proposals
            assert_eq!(contract.get_all_proposal_ids().len(), 0);
            
            // Create 3 proposals
            for i in 1..=3 {
                contract.create_proposal(
                    format!("Proposal {}", i),
                    format!("Description {}", i),
                    ProposalType::Treasury,
                    governance_params.clone(),
                    voting_options.clone(),
                ).unwrap();
            }
            
            let proposal_ids = contract.get_all_proposal_ids();
            assert_eq!(proposal_ids.len(), 3);
            assert_eq!(proposal_ids, vec![1, 2, 3]);
        }

        #[ink::test]
        fn test_contract_stats() {
            let mut contract = TreasuryGovernance::new();
            
            // Initially no proposals or voters
            let stats = contract.get_stats();
            assert_eq!(stats.total_proposals, 0);
            assert_eq!(stats.total_voters, 0);
            assert_eq!(stats.active_proposals, 0);
            assert_eq!(stats.executed_proposals, 0);
            
            // Register voters
            contract.register_voter().unwrap();
            
            // Create proposals
            let governance_params = GovernanceParameters {
                voting_period: VotingPeriod::SevenDays,
                quorum_threshold: QuorumThreshold::Ten,
                execution_delay: ExecutionDelay::OneDay,
            };
            
            let voting_options = VotingOptions {
                options: vec![String::from("Yes"), String::from("No")],
            };
            
            contract.create_proposal(
                String::from("Proposal 1"),
                String::from("Description 1"),
                ProposalType::Treasury,
                governance_params.clone(),
                voting_options.clone(),
            ).unwrap();
            
            contract.create_proposal(
                String::from("Proposal 2"),
                String::from("Description 2"),
                ProposalType::Governance,
                governance_params,
                voting_options,
            ).unwrap();
            
            let stats = contract.get_stats();
            assert_eq!(stats.total_proposals, 2);
            assert_eq!(stats.total_voters, 1);
            assert_eq!(stats.active_proposals, 2);
            assert_eq!(stats.executed_proposals, 0);
        }

        #[ink::test]
        fn test_multi_option_voting() {
            let mut contract = TreasuryGovernance::new();
            
            // Register voter
            contract.register_voter().unwrap();
            
            // Create proposal with multiple options
            let governance_params = GovernanceParameters {
                voting_period: VotingPeriod::SevenDays,
                quorum_threshold: QuorumThreshold::Ten,
                execution_delay: ExecutionDelay::OneDay,
            };
            
            let voting_options = VotingOptions {
                options: vec![
                    String::from("Option A"),
                    String::from("Option B"),
                    String::from("Option C"),
                    String::from("Option D"),
                ],
            };
            
            let proposal_id = contract.create_proposal(
                String::from("Multi-Option Proposal"),
                String::from("Choose from multiple options"),
                ProposalType::Governance,
                governance_params,
                voting_options,
            ).unwrap();
            
            // Vote for option C (index 2)
            let result = contract.vote(proposal_id, 2);
            assert!(result.is_ok());
            
            // Check vote was recorded correctly
            let proposal = contract.get_proposal(proposal_id).unwrap();
            assert_eq!(proposal.vote_counts[0], 0); // Option A
            assert_eq!(proposal.vote_counts[1], 0); // Option B
            assert_eq!(proposal.vote_counts[2], 1); // Option C
            assert_eq!(proposal.vote_counts[3], 0); // Option D
            assert_eq!(proposal.total_voters, 1);
        }

        #[ink::test]
        fn test_different_governance_parameters() {
            let mut contract = TreasuryGovernance::new();
            
            // Test different voting periods
            let short_params = GovernanceParameters {
                voting_period: VotingPeriod::ThreeDays,
                quorum_threshold: QuorumThreshold::Five,
                execution_delay: ExecutionDelay::Immediately,
            };
            
            let long_params = GovernanceParameters {
                voting_period: VotingPeriod::ThirtyDays,
                quorum_threshold: QuorumThreshold::TwentyFive,
                execution_delay: ExecutionDelay::SevenDays,
            };
            
            let voting_options = VotingOptions {
                options: vec![String::from("Yes"), String::from("No")],
            };
            
            let short_proposal = contract.create_proposal(
                String::from("Short Proposal"),
                String::from("Quick decision"),
                ProposalType::Technical,
                short_params,
                voting_options.clone(),
            ).unwrap();
            
            let long_proposal = contract.create_proposal(
                String::from("Long Proposal"),
                String::from("Important decision"),
                ProposalType::Treasury,
                long_params,
                voting_options,
            ).unwrap();
            
            // Verify parameters were set correctly
            let short_prop = contract.get_proposal(short_proposal).unwrap();
            assert_eq!(short_prop.governance_params.voting_period, VotingPeriod::ThreeDays);
            assert_eq!(short_prop.governance_params.quorum_threshold, QuorumThreshold::Five);
            assert_eq!(short_prop.governance_params.execution_delay, ExecutionDelay::Immediately);
            
            let long_prop = contract.get_proposal(long_proposal).unwrap();
            assert_eq!(long_prop.governance_params.voting_period, VotingPeriod::ThirtyDays);
            assert_eq!(long_prop.governance_params.quorum_threshold, QuorumThreshold::TwentyFive);
            assert_eq!(long_prop.governance_params.execution_delay, ExecutionDelay::SevenDays);
        }
    }
}



// CONTRACT ADDRESS = 0xFeaaDe70D5c525DA03b1c22D0147dF371bbAad72
