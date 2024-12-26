export interface RegisterAllNationsJob {
    gameId: string;
}

export interface UpdateNationInDBJob {
    // Required fields
    gameId: string;
    nationId: number;

    // Optional update fields
    authority?: string;
    mintedTokensTotal?: string;
    gdp?: string;
    healthcare?: string;
    environment?: string;
    stability?: string;
    gdpRewardRate?: string;
    healthcareRewardRate?: string;
    environmentRewardRate?: string;
    stabilityRewardRate?: string;
    isAlive?: boolean;
}

export interface CreatePlayerInDBJob {
    gameId: string;
    authority: string;
    xUsername: string;
}

export interface CreateStakedCitizenInDBJob {
    citizenAssetId: string;
    owner: string;
    nationId: number;
    gameId: string;
    completeSlot: number | bigint;
}

export interface DepositToBrokerJob {
    gameId: string;
    nationId: number;
    amount: number;
}

export interface CreateBountyInDBJob {
    gameId: string;
    bountyHash: string;
    amount: number;
    expirySlot: number;
}

export interface MintCitizenJob {
    gameId: string;
    playerAuthority: string;
    citizenAssetId: string;
    nationId: number;
    mintCost: number;
}
