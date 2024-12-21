import nacl from 'tweetnacl';
import bs58 from 'bs58';
import { PublicKey } from '@solana/web3.js';

const pkey = new PublicKey("Bz8Vbd2QeNK6NAQ2VPAKFLWSxc7yfQbW1nqJBitUY8Yi");

const presigned = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACAAQABAqM3cqMBKUytjrvqtA15hvRC1iRNDo/yzrdcVrNMb5vTAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAtSjQan5+TS+P2w0eiPAeoyUddrJCLl1uQ94HzkQ3r0wEBAgAADAIAAAABAAAAAAAAAAA="
const signature = "pfs9tDNScGXrGcyGMYRTFAGVf4mSJsw28NUZ1wUSHHQmeDwnGc4JYQsuqTSVzNi4bxuovJaCxvgqA4LicVAW6x8"
const sig2 = "KRvezreU6LE711hZkhcJ0/2VunP1daH3t9jhcw/C2q68u14eCmYI8ithD/wZH4cTqKmU7/7XWbPiHMiKk/CWCQ=="

const result = nacl.sign.detached.verify(
    Uint8Array.from(Buffer.from(presigned, "base64")),
    bs58.decode(signature),
    bs58.decode(pkey.toBase58()));
console.log("result: ", result);
const result2 = nacl.sign.detached.verify(
    Uint8Array.from(Buffer.from(presigned, "base64")),
    Uint8Array.from(Buffer.from(sig2, "base64")),
    bs58.decode(pkey.toBase58()))

console.log("result2: ", result2);