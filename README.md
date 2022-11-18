# Staking Options

Staking Options is a library for use in the Dual Finance ecosystem. One of the core beliefs of Dual Finance is that options are a strictly better incentive for projects than just token rewards. Options better align incentives between participants and the project. Staking Options is a program that helps projects convert existing incentives and create new pathways to leverage option based rewards.

## How to integrate for projects

1. config
	 - Project configures all the different parameters for the SO (Staking Option). This includes how long the options have until expiration, how many options are available, lot size, and other parameters needed for staking options.
2. initStrike
	 - Project decides what strike to configure for options. A project can customize strikes to unlock taregted value for there community. This is number of quote atoms per lot of base tokens.
3. issue
	 - Project calls into this program to issue options to a user. The project decides which strike to use and how many total options to give.
	 These are in units of lot size. All options are a call. The recipient can swap their quote tokens for base tokens. In order to make a put,
	 switch the base and quote and adjust lot/strike accordingly.
3. withdraw
	 - After the options expire, this program returns the remaining tokens to the project. The quote tokens were already given during exercise.

## Users
Users who receive options can go to [dual.finance](dual.finance) and exercise their options whenever they want before expiration. We are focused on delivering a market for the staking options themselves if users want to immediately sell their options to stablecoins, rather than hold them to expiration.
