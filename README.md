# Staking Options

Staking Options is a library for use in the Dual ecosystem. One of the core beliefs of Dual Labs is that options are a strictly better incentive for projects than just token rewards. Options better align incentives between participants and the project. Staking Options is a program that helps projects with converting their liquidity provider or other rewards into an option based reward.

## How to integrate for projects

 1. config
	 - Project configures all the different parameters for the SO (Staking Option). This includes how long the options have til expiration, how many options are available, and other parameters needed for staking options.
 2. initStrike
	 - Project decides what strike to configure for options. A project can do this for a variety of different strikes in order to make the different ways to incentivize users.
3. issue
	 - Project calls into this program to issue options to a user. The project decides which strike to use and how many options to give.
3. withdraw
	 - After the options expire, this program returns the remaining tokens to the project.

## Users
Users who receive options can go to [dual.finance](dual.finance) and exercise their options whenever they want before expiration. There will eventually be a market for the options themselves if users want to immediately sell their options and not deal with options.