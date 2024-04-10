import requests

url = 'http://127.0.0.1:8787' 

def ping_server():
    try:
        response = requests.get(url)
        if response.status_code == 200:
            print("Server is up and running")
        else:
            print(f"Server returned status code: {response.status_code}")
    except requests.ConnectionError:
        print("Failed to connect to the server")

def send_post_request():
    model = "gpt-3.5-turbo"
    messages = [
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "what's up homie!"}
    ]
    url = 'http://127.0.0.1:8787/chat/completions'
    headers = {'Content-Type': 'application/json'}
    payload = {
        "model": model,
        "messages": messages
    }

    try:
        response = requests.post(url, json=payload, headers=headers)
        if response.status_code == 200:
            print(f"POST request sent successfully with response: {response}")
        else:
            print(f"Failed to send POST request. Status code: {response.status_code}")
    except requests.ConnectionError:
        print(f"Failed to connect to the server.")

if __name__ == "__main__":
    ping_server()
    send_post_request()
