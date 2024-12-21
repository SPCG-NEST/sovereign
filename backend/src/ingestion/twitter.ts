import { Client } from 'twitter-api-sdk';
import dotenv from 'dotenv';

dotenv.config();
const client = new Client(process.env.TWITTER_BEARER_TOKEN!);
const ACCOUNTS_TO_MONITOR = ['@solana', '@aeyakovenko'];
const POLLING_INTERVAL_MS = 60000; // 1 minute
let lastCheckedId: string | null = null;

async function checkMentions() {
    try {
        // Create query string from accounts of interest
        const query = ACCOUNTS_TO_MONITOR
            .map(account => account.startsWith('@') ? account : `@${account}`)
            .join(' OR ');

        // Fetch tweets
        const response = await client.tweets.tweetsRecentSearch({
            query,
            "tweet.fields": ["author_id", "created_at"],
            "user.fields": ["username", "name"],
            "expansions": ["author_id"],
            ...(lastCheckedId && { since_id: lastCheckedId })
        });

        if (!response.data || response.data.length === 0) {
            console.log('No new mentions found');
            return;
        }

        // Update last checked id
        lastCheckedId = response.meta?.newest_id ?? lastCheckedId;

        // Create user lookup map
        const users = new Map(
            response.includes?.users?.map(user => [user.id, user]) || []
        );

        // Process tweets
        response.data.forEach((tweet) => {
            processTweet(tweet, users);
        });

    } catch (error) {
        console.log('Error:', JSON.stringify(error, null, 2));
    }
}

async function processTweet(tweet: any, users: Map<string, any>) {
    console.log('\n=== New Twitter mention ===');
    if (tweet.author_id) {
        const user = users.get(tweet.author_id);
        console.log(`From: @${user?.username} (${user?.name})`);
    }
    console.log(`Text: ${tweet.text}`);
    if (tweet.created_at) {
        console.log(`Time: ${new Date(tweet.created_at).toLocaleString()}`);
    }
}

// Start monitoring
console.log(`Starting to monitor mentions for: ${ACCOUNTS_TO_MONITOR.join(', ')}`);
console.log(`Polling interval: ${POLLING_INTERVAL_MS / 1000} seconds`);

checkMentions();
setInterval(checkMentions, POLLING_INTERVAL_MS);
