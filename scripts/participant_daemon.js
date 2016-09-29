/**
 * Randao participant daemon. Runs continuously monitoring a given Randao.
 * Partipates in Randao by submitting commits and reveals then withdrawing
 * any bounties at the end of a round.
 */

'use script'

const crypto = require('crypto');

const RandaoStage = {
    WAITING_COMMIT: 1,
    COMMIT: 2,
    REVEAL: 3,
    FINISHED: 4,
}
const CHECK_INTERVAL = 5000

let commitment, secret
let committed = revealed = false

/**
 * Check the current state of randao. If it is inside a commit or reveal round
 * and we havn't yet submitted a value then go ahead and submit. Otherwise do
 * nothing.
 *
 * @param web3 Connected instance of web3.js
 * @param randao Pudding abstraction for Randao.sol contract (truffle generated)
 * @param participantAddress Ethereum account address for the participant
 */

function doCheck(web3, randao, participantAddress) {
    const curBlk = web3.eth.blockNumber
    let cId, curCampaign
    randao.numCampaigns.call().then((numCampaigns) => {
        cId = numCampaigns - 1
        if (cId < 0) {
            console.warn(`No campaigns yet ...`)
            return
        }
        randao.campaigns.call(cId).then((campaign) => {
            const state = randaoState(campaign, curBlk)
            console.log(`state = ${state}`)
            switch (state) {
                case RandaoStage.COMMIT:
                    if (committed == false) {
                        console.log(`COMMIT`)
                        generateSecret(web3)
//                    commit(randao, commitment, participantAddress)
                        committed = true
                    }
                    break;
                case RandaoStage.REVEAL:
                    if (revealed == false) {
                        console.log(`REVEAL`)
//                    reveal(randao, secret, participantAddress)
                        revealed = true
                    }
                    break;
                case RandaoStage.WAITING_COMMIT:
                case RandaoStage.FINISHED:
                    committed = revealed = false
                    break;
                default:
                    console.error(`unknown state: ${state}`)
                    break;
            }
        })
    }).catch((e) => {
        console.error(`Error: ${e}`)
        throw e
    })
}


/**
 * Generate a 32 byte secret number using crypto randomBytes.
 */

function generateSecret(web3) {
    const buf = crypto.randomBytes(32);
    const hexStr = buf.toString('hex')
    console.log(`${hexStr}`)
    secret = web3.toDecimal(`0x${hexStr}`)
    console.log(`${secret}`)
    commitment = web3.sha3(hexStr, 'hex')
    console.log(`${commitment}`)
}


/**
 * Send commitment.
 *
 * @param randao Pudding abstraction for Randao.sol contract (truffle generated)
 * @param commitment Commitment - the SHA3 of the secret
 * @param accountAddress Address of participant
 */

function commit(randao, commitment, accountAddress) {
    randao.commit(commitment, {
        from: accountAddress
    }).then((tx) => {
        console.log(`commit done (tx:${tx})\n`)
    }).catch((e) => {
        console.error(`commit failed`)
        throw e
    })
}


/**
 * Send reveal.
 *
 * @param randao Pudding abstraction for Randao.sol contract (truffle generated)
 * @param reveal Secret reveal
 * @param accountAddress Address of participant
 */

function reveal(randao, reveal, accountAddress) {
    randao.reveal(reveal, {
        from: accountAddress
    }).then((tx) => {
        console.log(`reveal done (tx:${tx})\n`)
    }).catch((e) => {
        console.error(`reveal failed`)
        throw e
    })
}


/**
 * Determine current stage of Randao.
 *
 * @param campaignDetails Array of campaign details
 * @param curBlk Current block number on Ethereum
 * @return corresponding RandaoStage
 */

function randaoState(campaignDetails, curBlk) {
    const endBlk = campaignDetails[0]
    const balkline = campaignDetails[2]
    const deadline = campaignDetails[3]

    let state

    if (curBlk > endBlk)
        state = RandaoStage.FINISHED
    else if (curBlk >= endBlk - deadline)
        state = RandaoStage.REVEAL
    else if (curBlk >= endBlk - balkline)
        state = RandaoStage.COMMIT
    else
        state = RandaoStage.WAITING_COMMIT

    return state
}


/**
 * Randao participant deemon class. Provides just start() and stop().
 */

class ParticipantDaemon {

    /**
     * Constructor
     * @param web3 Connected instance of web3.js
     * @param randao Pudding abstraction for Randao.sol contract (truffle generated)
     * @param participantAddress Ethereum account address for the participant
     */

    constructor(web3, randao, participantAddress) {
        this.web3 = web3
        this.randao = randao
        this.participantAddress = participantAddress
    }

    /**
     * Start the daemon and set it to call doCheck every CHECK_INTERVAL milliseconds.
     */

    start() {
        this.intervalId = setInterval(doCheck, CHECK_INTERVAL,
            this.web3, this.randao, this.participantAddress)
    }

    /**
     * Stop the daemon.
     */

    stop() {
        cancelInterval(this.intervalId)
    }

}

module.exports = ParticipantDaemon
