/**
 * Randao campaign creation daemon. Runs continuously monitoring a given Randao.
 * If a campaign has ended it will create a new one.
 */

'use strict'

const utils = require('./utils')
const RandaoStage = utils.RandaoStage

const path = require('path')
const FILENAME = path.basename(__filename)

// TODO: move these to config
const CHECK_INTERVAL = 5000
const FULL_ROUND_LENGTH = 20 // blocks
// see Randao.newCampaign for an explanation of these:
const BALKLINE = 16
const DEADLINE = 8


/**
 * Campaign creation deemon class. Provides just start() and stop().
 */

class CampaignCreatorDaemon {

    /**
     * Constructor
     * @param web3 Connected instance of web3.js
     * @param randao Pudding abstraction for Randao.sol contract (truffle generated)
     * @param config Config properties
     */

    constructor(web3, randao, config) {
        this.web3 = web3
        this.randao = randao
        this.config = config
    }

    /**
     * Start the daemon and set it to call doCheck every CHECK_INTERVAL milliseconds.
     */

    start() {
        this.intervalId = setInterval(this.doCheck, CHECK_INTERVAL, this)
    }

    /**
     * Stop the daemon.
     */

    stop() {
        cancelInterval(this.intervalId)
    }

    /**
     * Check the current stage of randao. If there are no campaigns OR the most
     * recent campaign has finished then create a new one. Otherwise do nothing.
     */

    doCheck(self) {
        const curBlk = self.web3.eth.blockNumber
        let cId, curCampaign
        self.randao.numCampaigns.call().then((numCampaigns) => {
            cId = numCampaigns - 1
            if (cId < 0) {
                self.log(`No campaigns yet. Creating the first campaign ...`)
                self.newCampaign(self, curBlk)
                return
            }
            self.randao.campaigns.call(cId).then((campaign) => {
                if (curBlk > campaign[0]) {
                    self.log(`campaign ${cId} finished. Creating new campaign ...`)
                    self.newCampaign(self, curBlk)
                } else {
                    self.logCampaign(curBlk, cId, campaign)
                }
            })
        }).catch((e) => {
            self.logErr(`Error: ${e}`)
            throw e
        })
    }

    /**
     * Create a new campaign on the given randao.
     *
     * @param randao Pudding abstraction for Randao.sol contract (truffle generated)
     * @param curBlk Current block number on Ethereum
     */

    newCampaign(self, curBlk) {
        const randaoBlk = curBlk + FULL_ROUND_LENGTH
        self.randao.newCampaign(randaoBlk, self.config.deposit, BALKLINE, DEADLINE, {
            gas: 150000
        }).then((tx) => {
            self.log(`campaign created (tx:${tx})\n`)
        }).catch((e) => {
            self.logErr(`newCampaign failed`)
            throw e
        })
    }

    /**
     * Logging routines
     */

    logCampaign(curBlk, cId, campaign) {
        const stage = utils.getRandaoStage(curBlk, campaign[0], campaign[2], campaign[3])
        this.log(`block:${curBlk} campaign:${cId} stage:${RandaoStage.name(stage)} details:${campaign}`)
    }

    log(msg) {
        utils.log(`[${FILENAME}] ${msg}`)
    }

    logErr(msg) {
        utils.log(`[${FILENAME}] ${msg}`, true)
    }

}

module.exports = CampaignCreatorDaemon
