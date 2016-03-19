import Vivus from './lib/vivus'

let progress = {
  index: 0,
  vivus: null,
  init: function() {
    this.vivus = new Vivus('randao-progress', {
      type: 'async',
      duration: 150,
      start: 'manual'
    })
  },
  growth: function() {
    this.index += 0.0333
    this.vivus.setFrameProgress(this.index)
  }
}

export default progress
