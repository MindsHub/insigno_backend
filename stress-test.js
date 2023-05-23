import http from 'k6/http';
import { sleep } from 'k6';

export default function () {
    const authUrl = 'http://127.0.0.1:8000/performance';
    //sleep(Math.random());
    const params = {
        headers: {
         //'Content-Type': 'application/x-www-form-urlencoded',
         
        },
    };
    
    let formData = { name : 'test',
        password: 'PasswordDiTest1£',
        email: 'test@test.test',};
    let res = http.post(authUrl,formData,params);
    // launch with k6
}