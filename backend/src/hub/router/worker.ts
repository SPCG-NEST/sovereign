import type { Job } from "bullmq";
import * as Jobs from "../interfaces/jobs";
import { SVPRGM, DB, ACCOUNT_SEEDS, estimateInstructionSize, SIGNER_SIZE, BASE_TRANSACTION_SIZE, TRANSACTION_SIZE_LIMIT, CONNECTION, COMPUTE_UNIT_PRICE, estimateCU } from "../../common";
import bs58 from "bs58";
import { ComputeBudgetProgram, Keypair, PublicKey, Transaction, TransactionInstruction, TransactionMessage, VersionedTransaction, SystemProgram } from "@solana/web3.js";
import { ByteifyEndianess, serializeUint64, serializeUint8 } from "byteify";
import type { WorldLeader } from "../interfaces";
import { BN } from "@coral-xyz/anchor";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

export default async function worker(job: Job) {
    console.log(`Worker received job: ${job.id} - ${job.name}`);
    try {
        switch (job.name) {
            case "registerAllNations":
                await registerAllNations(job.data as Jobs.RegisterAllNationsJob);
                break;
            case "updateNationInDB":
                await updateNationInDB(job.data as Jobs.UpdateNationInDBJob);
                break;
            case "createPlayerInDB":
                await createPlayerInDB(job.data as Jobs.CreatePlayerInDBJob);
                break;
            case "mintCitizen":
                await mintCitizen(job.data as Jobs.MintCitizenJob);
                break;
            case "createStakedCitizenInDb":
                await createStakedCitizenInDB(job.data as Jobs.CreateStakedCitizenInDBJob);
                break;
            case "depositToBroker":
                await depositToBroker(job.data as Jobs.DepositToBrokerJob);
                break;
            case "createBountyInDB":
                await createBountyInDB(job.data as Jobs.CreateBountyInDBJob);
                break;
            default:
                throw new Error(`Unknown job: ${job.name}`);
        }
    } catch (e: any) {
        console.error(`Worker failed to process job: ${job.id} - ${job.name} - ${e.message}`);
    }
}

async function registerAllNations(job: Jobs.RegisterAllNationsJob) {
    console.log(`Registering all nations for game ${job.gameId}`);
    const nations = await DB.worldLeaders.findMany({});
    const gamekeys = await DB.game.findUnique({
        where: {
            gameId: BigInt(job.gameId),
        },
    });
    if (!gamekeys) {
        throw new Error(`Game keys not found: ${job.gameId}`);
    }
    const gameAuthority = Keypair.fromSecretKey(bs58.decode(gamekeys.adminPrivateKey));


    for (const nation of nations) {
        const worldLeader = nation.character as any as WorldLeader;
        const nationKey = Keypair.fromSecretKey(bs58.decode(nation.privateKey));
        const initNationArgs = {
            id: nation.nationId,
            gdp: new BN(worldLeader.stats.gdp),
            healthcare: new BN(worldLeader.stats.healthcare),
            environment: new BN(worldLeader.stats.environment),
            stability: new BN(worldLeader.stats.politicalStability),
            gdp_reward_rate: new BN(1),
            healthcare_reward_rate: new BN(1),
            environment_reward_rate: new BN(1),
            stability_reward_rate: new BN(1),
        }
        const nationAccountPubkey = PublicKey.findProgramAddressSync(
            [
                Buffer.from(ACCOUNT_SEEDS.NATION),
                Uint8Array.from(serializeUint64(gamekeys.gameId, { endianess: ByteifyEndianess.LITTLE_ENDIAN })),
                Uint8Array.from(serializeUint8(nation.nationId))
            ],
            SVPRGM.programId
        )[0]

        const ix = await SVPRGM.methods
            .initNation(initNationArgs)
            .accountsPartial({
                gameAuthority: gameAuthority.publicKey,
                nationAuthority: nationKey.publicKey,
                game: PublicKey.findProgramAddressSync(
                    [
                        Buffer.from(ACCOUNT_SEEDS.GAME),
                        Uint8Array.from(serializeUint64(gamekeys.gameId, { endianess: ByteifyEndianess.LITTLE_ENDIAN }))
                    ],
                    SVPRGM.programId
                )[0],
                nation: nationAccountPubkey
            })
            .signers([nationKey])
            .instruction();
        const tx = new VersionedTransaction(new TransactionMessage({
            payerKey: gameAuthority.publicKey,
            recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
            instructions: [
                ComputeBudgetProgram.setComputeUnitPrice({ microLamports: COMPUTE_UNIT_PRICE }),
                ComputeBudgetProgram.setComputeUnitLimit({ units: await estimateCU(gameAuthority.publicKey, [ix]) }),
                ix
            ]
        }).compileToV0Message());
        tx.sign([gameAuthority, nationKey]);
        const sig = await CONNECTION.sendTransaction(tx);
        console.log(`Registered nation ${nation.nationId} with sig ${sig}`);
        // Add the nations to the DB as well
        await DB.nationAccount.create({
            data: {
                pubkey: nationAccountPubkey.toBase58(),
                gameId: gamekeys.gameId,
                nationId: nation.nationId,
                authority: nationKey.publicKey.toBase58(),
                mintedTokensTotal: "0",
                gdp: new BN(worldLeader.stats.gdp).toString(),
                healthcare: new BN(worldLeader.stats.healthcare).toString(),
                environment: new BN(worldLeader.stats.environment).toString(),
                stability: new BN(worldLeader.stats.politicalStability).toString(),
                gdpRewardRate: "1",
                healthcareRewardRate: "1",
                environmentRewardRate: "1",
                stabilityRewardRate: "1",
                isAlive: true,
            }
        })
    }


    // TODO: Figure out batching for production (unkown signer error)
    /*
    let ixs = [];
    let ixNationSigners = [];
    for(const nation of nations) {
        const worldLeader = nation.character as any as WorldLeader;
        const nationKey = Keypair.fromSecretKey(bs58.decode(nation.privateKey));
        const initNationArgs = {
            id: nation.nationId,
            gdp: new BN(worldLeader.stats.gdp),
            healthcare: new BN(worldLeader.stats.healthcare),
            environment: new BN(worldLeader.stats.environment),
            stability: new BN(worldLeader.stats.politicalStability),
            gdp_reward_rate: new BN(1),
            healthcare_reward_rate: new BN(1),
            environment_reward_rate: new BN(1),
            stability_reward_rate: new BN(1),
        }
        ixNationSigners.push(nationKey);
        const ix = await SVPRGM.methods
            .initNation(initNationArgs)
            .accountsPartial({
                gameAuthority: gameAuthority.publicKey,
                nationAuthority: nationKey.publicKey,
                game: PublicKey.findProgramAddressSync(
                    [Buffer.from(ACCOUNT_SEEDS.GAME), Uint8Array.from(serializeUint64(gamekeys.gameId,{endianess: ByteifyEndianess.LITTLE_ENDIAN}))],
                    SVPRGM.programId
                )[0],
                nation: PublicKey.findProgramAddressSync(
                    [
                        Buffer.from(ACCOUNT_SEEDS.NATION),
                        Uint8Array.from(serializeUint64(gamekeys.gameId,{endianess: ByteifyEndianess.LITTLE_ENDIAN})),
                        Uint8Array.from(serializeUint8(nation.nationId))
                    ],
                    SVPRGM.programId
                )[0],
            })
            .signers([nationKey])
            .instruction();
        
        ixs.push(ix);
    }
    // 1 additional signer for each ix
    const initNationIxSize = estimateInstructionSize(ixs[0]) + SIGNER_SIZE;
    const recentBlockhash = (await CONNECTION.getLatestBlockhash()).blockhash;

    let transactions: Transaction[] = [];
    const buildInitNationTxn = (initNationIxs: TransactionInstruction[]): {txn: Transaction, remainingIxs: TransactionInstruction[]} => {
        let currentSize = BASE_TRANSACTION_SIZE;
        let txn = new Transaction();
        let i = 0;
        for(const ix of initNationIxs) {
            if(currentSize + initNationIxSize > TRANSACTION_SIZE_LIMIT) {
                break;
            }
            txn.add(ix);
            currentSize += initNationIxSize;
            i++;
        }
        txn.feePayer = gameAuthority.publicKey;
        txn.recentBlockhash = recentBlockhash;
        txn.sign(...ixNationSigners.slice(0, i), gameAuthority);
        return {txn, remainingIxs: initNationIxs.slice(i)};
    }
    let {txn, remainingIxs} = buildInitNationTxn(ixs);
    transactions.push(txn);
    while(remainingIxs.length > 0) {
        ({txn, remainingIxs} = buildInitNationTxn(remainingIxs));
        transactions.push(txn);
    }
    for(const txn of transactions) {
        txn.recentBlockhash = recentBlockhash;
        const sig = await CONNECTION.sendRawTransaction(txn.serialize());
        console.log(`Sent transaction ${sig}`);
    }

    console.log(`Registered ${nations.length} nations in ${transactions.length} transactions`);
    */
}

export async function updateNationInDB(job: Jobs.UpdateNationInDBJob) {
    console.log(`Updating nation ${job.nationId} in game ${job.gameId}`);

    try {
        const { gameId, nationId, ...updateFields } = job;
        const nationAccountPubkey = PublicKey.findProgramAddressSync(
            [
                Buffer.from(ACCOUNT_SEEDS.NATION),
                Uint8Array.from(serializeUint64(BigInt(gameId), { endianess: ByteifyEndianess.LITTLE_ENDIAN })),
                Uint8Array.from(serializeUint8(nationId))
            ],
            SVPRGM.programId
        )[0];

        const updatedNation = await DB.nationAccount.update({
            where: {
                pubkey: nationAccountPubkey.toBase58()
            },
            data: updateFields,
        });

        console.log(`Successfully updated nation ${nationId} in game ${gameId}`);
        return updatedNation;

    } catch (error) {
        if (error instanceof Error) {
            throw new Error(`Failed to update nation in DB: ${error.message}`);
        }
        throw error;
    }
}

export async function createPlayerInDB(job: Jobs.CreatePlayerInDBJob) {
    console.log(`Creating player account for authority ${job.authority} in game ${job.gameId}`);

    try {
        const playerAccountPubkey = PublicKey.findProgramAddressSync(
            [
                Buffer.from(ACCOUNT_SEEDS.PLAYER),
                Uint8Array.from(serializeUint64(BigInt(job.gameId), { endianess: ByteifyEndianess.LITTLE_ENDIAN })),
                new PublicKey(job.authority).toBuffer()
            ],
            SVPRGM.programId
        )[0];

        const player = await DB.playerAccount.create({
            data: {
                pubkey: playerAccountPubkey.toBase58(),
                gameId: BigInt(job.gameId),
                authority: job.authority,
                xUsername: job.xUsername
            }
        });

        console.log(`Created player account ${playerAccountPubkey.toBase58()} for authority ${job.authority}`);
        return player;

    } catch (error) {
        if (error instanceof Error) {
            throw new Error(`Failed to create player in DB: ${error.message}`);
        }
        throw error;
    }
}

export async function createStakedCitizenInDB(job: Jobs.CreateStakedCitizenInDBJob) {
    console.log(`Creating staked citizen account for citizen ${job.citizenAssetId} in game ${job.gameId}`);
    try {
        const stakedCitizenPubkey = PublicKey.findProgramAddressSync(
            [
                Buffer.from("staked_citizen"),
                Uint8Array.from(serializeUint64(BigInt(job.gameId), { endianess: ByteifyEndianess.LITTLE_ENDIAN })),
                Uint8Array.from([job.nationId]),
                new PublicKey(job.citizenAssetId).toBuffer()
            ],
            SVPRGM.programId
        )[0];

        const stakedCitizen = await DB.stakedCitizenAccount.create({
            data: {
                pubkey: stakedCitizenPubkey.toBase58(),
                citizenAssetId: job.citizenAssetId,
                owner: job.owner,
                nationId: job.nationId,
                gameId: BigInt(job.gameId),
                rewardAmount: '0',
                completeSlot: job.completeSlot.toString()
            }
        });

        console.log(`Created staked citizen account ${stakedCitizenPubkey.toBase58()} for citizen ${job.citizenAssetId}`);
        return stakedCitizen;
    } catch (error) {
        if (error instanceof Error) {
            throw new Error(`Failed to create staked citizen in DB: ${error.message}`);
        }
        throw error;
    }
}

export async function depositToBroker(job: Jobs.DepositToBrokerJob) {
    // TODO
    // // Create the game authority keypair
    // const gameData = await DB.game.findUnique({
    //     where: {
    //         gameId: BigInt(job.gameId),
    //     },
    // });
    // if (!gameData) {
    //     throw new Error(`Game not found: ${job.gameId}`);
    // }
    // const gameAuthority = Keypair.fromSecretKey(bs58.decode(gameData.adminPrivateKey));

    // // Convert bounty hash string to bytes
    // const bountyHashBytes = Buffer.from(job.bountyHash, 'hex');
    // if (bountyHashBytes.length !== 32) {
    //     throw new Error('Bounty hash must be 32 bytes');
    // }

    // const gamePDA = PublicKey.findProgramAddressSync(
    //     [
    //         Buffer.from(ACCOUNT_SEEDS.GAME),
    //         Uint8Array.from(serializeUint64(gameData.gameId, { endianess: ByteifyEndianess.LITTLE_ENDIAN }))
    //     ],
    //     SVPRGM.programId
    // )[0];
    // const bountyPDA = PublicKey.findProgramAddressSync(
    //     [
    //         Buffer.from(ACCOUNT_SEEDS.BOUNTY),
    //         Uint8Array.from(serializeUint64(gameData.gameId, { endianess: ByteifyEndianess.LITTLE_ENDIAN })),
    //         bountyHashBytes
    //     ],
    //     SVPRGM.programId
    // )[0];

    // // Create and send the transaction
    // const ix = await SVPRGM.methods
    //     .createBounty({
    //         bountyHash: Array.from(bountyHashBytes),
    //         amount: new BN(job.amount),
    //         expirySlot: new BN(job.expirySlot)
    //     })
    //     .accountsPartial({
    //         brokerKey: gameAuthority.publicKey,
    //         game: gamePDA,
    //         bounty: bountyPDA,
    //         systemProgram: SYSTEM_PROGRAM_ID
    //     })
    //     .instruction();

    // const tx = new VersionedTransaction(new TransactionMessage({
    //     payerKey: gameAuthority.publicKey,
    //     recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
    //     instructions: [
    //         ComputeBudgetProgram.setComputeUnitPrice({ microLamports: COMPUTE_UNIT_PRICE }),
    //         ComputeBudgetProgram.setComputeUnitLimit({ units: await estimateCU(gameAuthority.publicKey, [ix]) }),
    //         ix
    //     ]
    // }).compileToV0Message());

    // tx.sign([gameAuthority]);
    // const sig = await CONNECTION.sendTransaction(tx);
    // console.log(`Registered bounty with sig ${sig}`);
}

export async function createBountyInDB(job: Jobs.CreateBountyInDBJob) {
    console.log(`Creating bounty account with hash ${job.bountyHash} in game ${job.gameId}`);
    try {
        const bountyPubkey = PublicKey.findProgramAddressSync(
            [
                Buffer.from(ACCOUNT_SEEDS.BOUNTY),
                Uint8Array.from(serializeUint64(BigInt(job.gameId), { endianess: ByteifyEndianess.LITTLE_ENDIAN })),
                Buffer.from(job.bountyHash, 'hex')
            ],
            SVPRGM.programId
        )[0];

        const bounty = await DB.bountyAccount.create({
            data: {
                pubkey: bountyPubkey.toBase58(),
                gameId: BigInt(job.gameId),
                bountyHash: job.bountyHash,
                amount: job.amount.toString(),
                expirySlot: job.expirySlot.toString()
            }
        });

        console.log(`Created bounty account ${bountyPubkey.toBase58()} with hash ${job.bountyHash}`);
        return bounty;
    } catch (error) {
        if (error instanceof Error) {
            throw new Error(`Failed to create bounty in DB: ${error.message}`);
        }
        throw error;
    }
}

async function mintCitizen(job: Jobs.MintCitizenJob) {
    console.log(`Processing mint payment for citizen ${job.citizenAssetId} for game ${job.gameId}`);

    const gamekeys = await DB.game.findUnique({
        where: { gameId: BigInt(job.gameId) }
    });

    if (!gamekeys) {
        throw new Error(`Game keys not found: ${job.gameId}`);
    }

    const nationAccount = await DB.nationAccount.findFirst({
        where: {
            gameId: BigInt(job.gameId),
            nationId: job.nationId,
        }
    });

    if (!nationAccount) {
        throw new Error(`Nation account not found for game ${job.gameId} and nation ${job.nationId}`);
    }

    const worldAuthority = Keypair.fromSecretKey(bs58.decode(gamekeys.worldAgentPrivateKey));
    const gameIdBuffer = Uint8Array.from(
        serializeUint64(gamekeys.gameId, { endianess: ByteifyEndianess.LITTLE_ENDIAN })
    );

    const gamePda = PublicKey.findProgramAddressSync(
        [
            Buffer.from(ACCOUNT_SEEDS.GAME),
            gameIdBuffer
        ],
        SVPRGM.programId
    )[0];

    const worldAgentWalletPda = PublicKey.findProgramAddressSync(
        [
            Buffer.from(ACCOUNT_SEEDS.WALLET),
            gameIdBuffer,
            worldAuthority.publicKey.toBytes()
        ],
        SVPRGM.programId
    )[0];

    // Nation recieves 80% share of mint cost
    const nationShare = new BN(job.mintCost * 0.8);
    const ix = await SVPRGM.methods
        .nationBoost({
            lamportsAmount: nationShare
        })
        .accountsPartial({
            worldAuthority: worldAuthority.publicKey,
            worldAgentWallet: worldAgentWalletPda,
            gameAccount: gamePda,
            nation: new PublicKey(nationAccount.pubkey),
            nationAuthority: new PublicKey(nationAccount.authority)
        })
        .instruction();

    const tx = new VersionedTransaction(new TransactionMessage({
        payerKey: worldAuthority.publicKey,
        recentBlockhash: (await CONNECTION.getLatestBlockhash()).blockhash,
        instructions: [
            ComputeBudgetProgram.setComputeUnitPrice({ microLamports: COMPUTE_UNIT_PRICE }),
            ComputeBudgetProgram.setComputeUnitLimit({ units: await estimateCU(worldAuthority.publicKey, [ix]) }),
            ix
        ]
    }).compileToV0Message());
    tx.sign([worldAuthority]);

    const sig = await CONNECTION.sendTransaction(tx);
    console.log(`Distributed ${nationShare.toString()} lamports to nation ${job.nationId} with sig ${sig}`);
}