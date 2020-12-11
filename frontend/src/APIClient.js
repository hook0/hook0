const axios = require('axios');
const Promise = require('bluebird');

const http = axios.create({
  baseURL: '/api/v1',
  timeout: 1000,
  headers: {
    /*'X-Custom-Header': 'foobar'*/
  },
});

module.exports = {
  applications: {
    list() {
      return http
        .request({
          method: 'get',
          url: '/applications',
          data: {},
        })
        .then(res => res.data);
    },
    add(application) {},
    get(application_id) {},
    edit(application_id, body) {},
    delete(application_id) {},
  },
};
