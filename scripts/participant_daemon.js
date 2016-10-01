/**
 * Randao participant daemon. Runs continuously monitoring a given Randao.
 * Participates in Randao by submitting commits and reveals when they are
 * required. Finally it will withdrawing any bounties at the end of a round.
 */

'use strict'

const utils = require('./utils')
const RandaoStage = utils.RandaoStage

const path = require('path')
const FILENAME = path.basename(__filename)

// TODO: move to config
const CHECK_INTERVAL = 5000


/**
 * Randao participant deemon class. Provides just start() and stop().
 */

class ParticipantDaemon {

    /**
     * Constructor
     * @param web3 Connected instance of web3.js
     * @param randao Pudding abstraction for Randao.sol contract (truffle generated)
     * @param config Config properties
     * @param participantAddress Ethereum account address for the participant
     */

    constructor(web3, randao, config, participantAddress) {
        this.web3 = web3
        this.randao = randao
        this.config = config
        this.participantAddress = participantAddress
        this.pAddrShort = `${participantAddress.substr(0, 6)}...`
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
     * Check the current stage of randao. If it is inside a commit or reveal round
     * and we havn't yet submitted a value then go ahead and submit. Otherwise do
     * nothing.
     */

    doCheck(self) {
        const curBlk = self.web3.eth.blockNumber

        let cId, curCampaign
        self.randao.numCampaigns.call().then((numCampaigns) => {
            cId = numCampaigns - 1
            if (cId < 0) {
                self.log(`No campaigns yet ...`)
                return
            }
            self.randao.campaigns.call(cId).then((campaign) => {
                const stage = utils.getRandaoStage(curBlk, campaign[0], campaign[2], campaign[3])
                switch (stage) {
                    case RandaoStage.COMMIT:
                        if (self.committed == false) {
                            self.generateSecret()
                            self.commit(self)
                        }
                        break;

                    case RandaoStage.REVEAL:
                        if (self.revealed == false) {
                            self.reveal(self)
                        }
                        break;

                    case RandaoStage.WAITING_COMMIT:
                    case RandaoStage.FINISHED:
                        self.committed = self.revealed = false
                        break;

                    default:
                        self.logErr(`unknown stage: ${stage}`)
                        break;
                }
            })
        }).catch((e) => {
            self.logErr(`Error: ${e}`)
            throw e
        })
    }

    /**
     * Send commitment.
     */

    commit(self) {
        self.log('sendTx commit()')
        self.randao.commit(self.commitment, {
            from: self.participantAddress,
            value: self.config.deposit
        }).then((tx) => {
            self.log(`commit done (tx:${tx})\n`)
            self.committed = true
        }).catch((e) => {
            self.logErr(`commit failed`)
            throw e
        })
    }

    /**
     * Send reveal.
     */

    reveal(self) {
        self.log('sendTx reveal()')
        self.randao.reveal(self.secret, {
            from: self.participantAddress
        }).then((tx) => {
            self.log(`reveal done (tx:${tx})\n`)
            self.revealed = true
        }).catch((e) => {
            self.logErr(`reveal failed`)
            throw e
        })
    }

    /**
     * Create secret and corresponding commitment
     */

    generateSecret() {
        this.log('Generating secret for commit ...')
        const hexStr = utils.random32()
        this.secret = this.web3.toDecimal(`0x${hexStr}`)
        this.commitment = this.web3.sha3(hexStr, 'hex')
    }

    /**
     * Logger routines
     */

    log(msg) {
        utils.log(`[${FILENAME}] [${this.pAddrShort}] ${msg}`)
    }

    logErr(msg) {
        utils.log(`[${FILENAME}] [${this.pAddrShort}] ${msg}`, true)
    }

}

module.exports = ParticipantDaemon
