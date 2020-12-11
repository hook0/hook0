const components = {
  Logo: require('./Logo.vue').default,
  HelloWorld: require('./HelloWorld.vue').default,
  MenuItem: require('./MenuItem.vue').default,
  Icon: require('./Icon.vue').default,
};

module.exports = function(Vue) {
  Object.entries(components).forEach(([key, value]) => Vue.component(key, value));
};

module.exports.components = components;
