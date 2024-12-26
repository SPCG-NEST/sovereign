import { z } from 'zod';

export interface SolanaEvent {
    txn: string;
    name: string;
    data: any;
}

export interface AccountUpdate {
    context: {
        slot: string
    },
    value: {
        pubkey: string,
        account: {
            lamports: string,
            owner: string,
            data: string,
            executable: boolean,
            rentEpoch: string,
            space: string,
        }
    }
}

export const NewGameEvent = z.object({
    gameId: z.string(),
    authority: z.string(),
});

export const MintTokensToPlayerWalletEvent = z.object({
    gameId: z.string(),
    nationId: z.number(),
    playerAuthority: z.string(),
    playerWallet: z.string(),
    amount: z.string(),
    slot: z.string(),
});

export const DepositToBrokerEvent = z.object({
    gameId: z.string(),
    nationId: z.number(),
    amount: z.string(),
});

export const CoupNationEvent = z.object({
    gameId: z.string(),
});

export const ClaimBountyEvent = z.object({
    gameId: z.string(),
    bountyHash: z.string(),
    playerAuthority: z.string(),
    amount: z.string(),
    citizenAssetId: z.string(),
});

export const MintCitizenEvent = z.object({
    gameId: z.string(),
    playerAuthority: z.string(),
    citizenAssetId: z.string(),
    nationId: z.number(),
    mintCost: z.number(),
});

export const WorldDisasterEvent = z.object({
    gameId: z.string(),
    nationId: z.number(),
    gdpDamage: z.string(),
    healthDamage: z.string(),
    environmentDamage: z.string(),
    stabilityDamage: z.string(),
});

export const NationBoostEvent = z.object({
    gameId: z.string(),
    nationId: z.number(),
    lamportsAmount: z.string(),
});

export const StakeCitizenEvent = z.object({
    gameId: z.string(),
    owner: z.string(),
    citizenAssetId: z.string(),
    nationId: z.number(),
    complete_slot: z.string(),
});

export const CompleteStakeEvent = z.object({
    gameId: z.string(),
    playerAuthority: z.string(),
    citizenAssetId: z.string(),
    nationId: z.number(),
    rewardAmount: z.number(),
    slot: z.string(),
    nation_gdp: z.number(),
    nation_healthcare: z.number(),
    nation_environment: z.number(),
    nation_stability: z.number(),
});

export const NationDissolutionEvent = z.object({
    gameId: z.string(),
    nationId: z.number(),
});

export const UpdateNationRewardRateEvent = z.object({
    gameId: z.string(),
    nationId: z.number(),
    gdpRewardRate: z.number(),
    healthcareRewardRate: z.number(),
    environmentRewardRate: z.number(),
    stabilityRewardRate: z.number(),
});

export const RegisterPlayerEvent = z.object({
    gameId: z.string(),
    playerAuthority: z.string(),
    xUsername: z.string(),
});

export const RegisterBountyEvent = z.object({
    gameId: z.string(),
    nationId: z.number(),
    bountyHash: z.string(),
    amount: z.number(),
    expirySlot: z.number(),
});

export const CreateBountyEvent = z.object({
    gameId: z.string(),
    bountyHash: z.string(),
    amount: z.number(),
    expirySlot: z.number(),
});

export const LootNationEvent = z.object({
    gameId: z.string(),
    nationId: z.number(),
    playerAuthority: z.string(),
    amount: z.string(),
});
