const locals = {};

locals.meta = require('./data/meta');
locals.social = require('./data/social');
locals.prices = require('./data/prices');
locals.features = require('./data/features')(locals);

module.exports = {
  "locals": locals
}
