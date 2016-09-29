/**
 * Randao campaign creation daemon. Runs continuously monitoring a given Randao.
 * If a campaign has ended it will create a new one.
 */

'use script'

const CHECK_INTERVAL = 5000
const DEPOSIT = 1000
const FULL_ROUND_LENGTH = 20 // blocks

// see Randao.newCampaign for an explanation of these:
const BALKLINE = 16
const DEADLINE = 8


/**
 * Check the current state of randao. If there are no campaigns yet or the most
 * recent campaign has finished then create a new one. Otherwise do nothing.
 *
 * @param web3 Connected instance of web3.js
 * @param randao Pudding abstraction for Randao.sol contract (truffle generated)
 */

function doCheck(web3, randao) {
    const curBlk = web3.eth.blockNumber
    let cId, curCampaign
    randao.numCampaigns.call().then((numCampaigns) => {
        cId = numCampaigns - 1
        if (cId < 0) {
            console.warn(`No campaigns yet. Creating the first campaign ...`)
            newCampaign(randao, curBlk)
            return
        }
        randao.campaigns.call(cId).then((campaign) => {
            if (curBlk > campaign[0]) {
                console.log(`campaign ${cId} finished. Creating new campaign ...`)
                newCampaign(randao, curBlk)
            } else {
                logCampaign(curBlk, cId, campaign)
            }
        })
    }).catch((e) => {
        console.error(`Error: ${e}`)
        throw e
    })
}


/**
 * Create a new campaign on the given randao.
 *
 * @param randao Pudding abstraction for Randao.sol contract (truffle generated)
 * @param curBlk Current block number on Ethereum
 */

function newCampaign(randao, curBlk) {
    const randaoBlk = curBlk + FULL_ROUND_LENGTH
    randao.newCampaign(randaoBlk, DEPOSIT, BALKLINE, DEADLINE, {
        gas: 150000
    }).then((tx) => {
        console.log(`campaign created (tx:${tx})\n`)
    }).catch((e) => {
        console.error(`newCampaign failed`)
        throw e
    })
}


/**
 * Create a new campaign on the given randao.
 *
 * @param curBlk Current Ethereum block number
 * @param cId Randao current campaign id
 * @param campaign Array of campaign details (see struct Campaign in contract)
 */

function logCampaign(curBlk, cId, campaign) {
    console.log(`${new Date().toISOString()}: block:${curBlk} campaign:${cId} details:${campaign}`)
}


/**
 * Campaign creation deemon class. Provides just start() and stop().
 */

class CampaignCreatorDaemon {

    /**
     * Constructor
     * @param web3 Connected instance of web3.js
     * @param randao Pudding abstraction for Randao.sol contract (truffle generated)
     */

    constructor(web3, randao) {
        this.web3 = web3
        this.randao = randao
    }

    /**
     * Start the daemon and set it to call doCheck every CHECK_INTERVAL milliseconds.
     */

    start() {
        this.intervalId = setInterval(doCheck, CHECK_INTERVAL, this.web3, this.randao)
    }

    /**
     * Stop the daemon.
     */

    stop() {
        cancelInterval(this.intervalId)
    }

}

module.exports = CampaignCreatorDaemon
