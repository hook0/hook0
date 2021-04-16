import { AxiosResponse } from 'axios';
import axios from 'axios';

require('bluebird');
const http = axios.create({
  baseURL: 'http://localhost:8080/api/v1',
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
        .then((res: AxiosResponse<any>) => res.data);
    },
    add(application: any) {},
    get(application_id: any) {},
    edit(application_id: any, body: any) {},
    delete(application_id: any) {},
  },
};
