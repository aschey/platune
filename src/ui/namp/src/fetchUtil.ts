import process from 'process';
import { rejects } from 'assert';
console.log(process.env.NODE_ENV);
const options = {
    headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json;charset=UTF-8'
    }
};

const success = async <T>(response: Response): Promise<T> => {
    if (!response.ok) {
        if (response.status === 400) {
            const res = await response.text();
            throw new Error(res);
        }
        throw new Error("An error occurred");
    }
    const data: T = await response.json();
    return data;
}

const baseUrl = 'http://localhost:5000';
export const getJson = <T>(url: string): Promise<T> => fetch(baseUrl + url, {method: 'GET', ...options})
    .then(async response => await success<T>(response));
    //.catch(async err => await Promise.reject(error(err)));

export const putJson = <T>(url: string, body: {}): Promise<T> => fetch(baseUrl + url, {method: 'PUT', body: JSON.stringify(body), ...options})
    .then(async response => await success<T>(response));
    //.catch(async err => await Promise.reject(error(err)));
