import Redis from 'ioredis';
import { REDIS_CHANNELS } from '../../common';
import type { SERVICE } from '../interfaces';
import { Queue, Worker } from 'bullmq';
import * as SolanaEvents from '../interfaces/solana';
import * as Jobs from '../interfaces/jobs';
import worker from './worker';

const REMOVE_OPTS = {
    removeOnComplete: {
        age: 3600, //keep up to 1 hour
        count: 100, //keep up to 100 jobs
    },
    removeOnFail: {
        age: 3600,
        count: 100,
    },
}

const WORKER_COUNT = 10;

export class Router implements SERVICE {
    private receiver: Redis;
    private router: Redis;
    private JOBS_QUEUE: Queue;
    private WORKERS: Worker[];

    constructor() {
        this.router = new Redis(process.env.REDIS_URL || "redis://localhost:6379", { maxRetriesPerRequest: null });
        this.receiver = new Redis(process.env.REDIS_URL || "redis://localhost:6379", { maxRetriesPerRequest: null });
        this.JOBS_QUEUE = new Queue(REDIS_CHANNELS.JOBS_QUEUE, { connection: this.router });
        this.JOBS_QUEUE.setGlobalConcurrency(10); //todo: experiment with how many jobs can be run at once
        this.WORKERS = [];
        for (let i = 0; i < WORKER_COUNT; i++) {
            this.WORKERS.push(new Worker(REDIS_CHANNELS.JOBS_QUEUE, worker, { concurrency: 5, connection: this.router }));
        }
    }

    async start() {
        // Subscribe to EVENTS_QUEUE
        // For each event, determine RECIEVERS
        // For each RECIEVER, add a JOB to the JOBS_QUEUE
        console.log(`Router listening on ${this.router.options.host}:${this.router.options.port}`);
        try {
            await this.receiver.subscribe(REDIS_CHANNELS.EVENTS_QUEUE, () => {
                console.log(`Router subscribed to ${REDIS_CHANNELS.EVENTS_QUEUE}`);

                this.receiver.on('message', (channel, message) => {
                    console.log(`Received event: ${message}`);
                    this.createJobs(JSON.parse(message));
                });
            });
        } catch (e: any) {
            console.error(`Error in router: ${e.message}`);
        }
    }

    async createJobs(event: { name: string, data: any, txn: string }) {
        // For each event, determine RECIEVERS
        // For each RECIEVER, add a JOB to the JOBS_QUEUE
        try {
            switch (event.name) {
                case "newGameEvent":
                    const newGameEvt = SolanaEvents.NewGameEvent.parse(event.data);
                    // This is a unique event where we need to batch register all nations
                    // Normally we'd fire a job per agent, but here we just want the worker to take care of it
                    // No need to invoke any agents 
                    await this.JOBS_QUEUE.add(
                        "registerAllNations",
                        {
                            gameId: newGameEvt.gameId,
                            authority: newGameEvt.authority,
                        } as Jobs.RegisterAllNationsJob,
                        REMOVE_OPTS
                    );
                    break;
                case "updateNationRewardRateEvent":
                    await this.JOBS_QUEUE.add(
                        "updateNationInDB",
                        {
                            gameId: event.data.gameId,
                            nationId: event.data.nationId,
                            gdpRewardRate: event.data.gdpRewardRate,
                            healthcareRewardRate: event.data.healthcareRewardRate,
                            environmentRewardRate: event.data.environmentRewardRate,
                            stabilityRewardRate: event.data.stabilityRewardRate,
                        } as Jobs.UpdateNationInDBJob,
                        REMOVE_OPTS
                    );
                    // TODO: NationUpdate Job for Journalist
                    break;
                case "registerPlayerEvent":
                    await this.JOBS_QUEUE.add(
                        "createPlayerInDB",
                        {
                            gameId: event.data.gameId,
                            authority: event.data.playerAuthority,
                            xUsername: event.data.xUsername,
                        } as Jobs.CreatePlayerInDBJob,
                        REMOVE_OPTS
                    );
                    // TODO: (Journalist JOB) (in case someone with large number of followers joined)
                    break;
                case "mintCitizenEvent":
                    await this.JOBS_QUEUE.add(
                        "mintCitizen",
                        {
                            gameId: event.data.gameId,
                            playerAuthority: event.data.playerAuthority,
                            citizenAssetId: event.data.citizenAssetId,
                            nationId: event.data.nationId,
                            mintCost: event.data.mintCost,
                        } as Jobs.MintCitizenJob,
                        REMOVE_OPTS
                    );
                    break;
                case "stakeCitizenEvent":
                    await this.JOBS_QUEUE.add(
                        "createStakedCitizenInDB",
                        {
                            gameId: event.data.gameId,
                            owner: event.data.owner,
                            citizenAssetId: event.data.citizenAssetId,
                            nationId: event.data.nationId,
                            completeSlot: event.data.completeSlot,
                        } as Jobs.CreateStakedCitizenInDBJob,
                        REMOVE_OPTS
                    );
                    // TODO: (Let nation know that a citizen has been staked to it)
                    // TODO: (Journalist JOB)
                    break;
                case "completeStakeEvent":
                    await this.JOBS_QUEUE.add(
                        "updateNationInDB",
                        {
                            gameId: event.data.gameId,
                            nationId: event.data.nationId,
                            gdp: event.data.nationGdp,
                            healthcare: event.data.nationHealthcare,
                            environment: event.data.nationEnvironment,
                            stability: event.data.nationStability,
                        } as Jobs.UpdateNationInDBJob,
                        REMOVE_OPTS
                    );
                    // TODO: (Let nation know)
                    // TODO: (Journalist JOB)
                    break;
                case "worldDisasterEvent":
                    await this.JOBS_QUEUE.add(
                        "updateNationInDB",
                        {
                            gameId: event.data.gameId,
                            nationId: event.data.nationId,
                            gdp: event.data.nationGdp,
                            healthcare: event.data.nationHealthcare,
                            environment: event.data.nationEnvironment,
                            stability: event.data.nationStability,
                        } as Jobs.UpdateNationInDBJob,
                        REMOVE_OPTS
                    );
                    // TODO: (Let nation know)
                    // TODO: (Journalist JOB)
                    break
                case "nationBoostEvent":
                    // TODO (Let nation know)
                    // TODO (Journalist JOB)
                    break;
                case "depositToBrokerEvent":
                    // Event for when the nation wants the broker to create an onchain bounty
                    await this.JOBS_QUEUE.add(
                        "depositToBroker",
                        {
                            gameId: event.data.gameId,
                            nationId: event.data.nationId,
                            amount: event.data.amount,
                        } as Jobs.DepositToBrokerJob,
                        REMOVE_OPTS
                    );
                    break;
                case "createBountyEvent":
                    // Event for when the broker has created an onchain bounty
                    await this.JOBS_QUEUE.add(
                        "createBountyInDB",
                        {
                            gameId: event.data.gameId,
                            bountyHash: event.data.bountyHash,
                            amount: event.data.amount,
                            expirySlot: event.data.expirySlot,
                        } as Jobs.CreateBountyInDBJob,
                        REMOVE_OPTS
                    );
                    break;
                case "claimBountyEvent":
                    // TODO (Let nation(s?) know)
                    // TODO (Journalist JOB)
                    break;
                case "mintTokensToPlayerWalletEvent":
                    break;
                case "nationDissolutionEvent":
                    break;
                case "coupNationEvent":
                    break;
                case "lootNationEvent":
                    break;
                default:
                    throw new Error(`Unknown event: ${event.name}`);
            }
        } catch (e: any) {
            console.error(`Error in createJobs: ${e.message}`);
        }
    }
}