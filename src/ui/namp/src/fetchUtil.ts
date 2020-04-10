
const options = {
    headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json;charset=UTF-8'
    }
};

const baseUrl = 'http://localhost:5000';
export const getJson = <T>(url: string): Promise<T> => fetch(baseUrl + url, {method: 'GET', ...options})
    .then(async response => {
        const data: T = await response.json();
        return data;
    });

export const putJson = <T>(url: string, body: {}): Promise<T> => fetch(baseUrl + url, {method: 'PUT', body: JSON.stringify(body), ...options})
    .then(async response => {
        const data: T = await response.json();
        return data;
    });