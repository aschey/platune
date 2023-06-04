const options = {
  headers: {
    Accept: 'application/json',
    'Content-Type': 'application/json;charset=UTF-8',
  },
};

const success = async <T>(response: Response): Promise<T> => {
  if (!response.ok) {
    if (response.status === 400) {
      const res = await response.text();
      throw new Error(res);
    }
    console.log(response);
    throw new Error('An error occurred');
  }
  const data: T = await response.json();
  return data;
};

const baseUrl = 'http://localhost:5000';

export const getJson = async <T>(url: string) => {
  const response = await fetch(baseUrl + url, { method: 'GET', ...options });
  return await success<T>(response);
};

export const deleteJson = async <T>(url: string) => {
  const response = await fetch(baseUrl + url, { method: 'DELETE', ...options });
  return await success<T>(response);
};

export const putJson = async <T>(url: string, body: {}) => {
  const response = await fetch(baseUrl + url, { method: 'PUT', body: JSON.stringify(body), ...options });
  return await success<T>(response);
};

export const postJson = async <T>(url: string, body: {}): Promise<T> => {
  const response = await fetch(baseUrl + url, { method: 'POST', body: JSON.stringify(body), ...options });
  return await success<T>(response);
};
