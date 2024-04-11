import requests
import sys

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
    # Check if a command-line argument is provided
    if len(sys.argv) > 1:
        user_content = sys.argv[1]  # Use the first command-line argument
    else:
        user_content = "Default message"  # Fallback message
    
    messages = [
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": user_content}
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
            print(f"POST request sent successfully with response: {response.json()}")
        else:
            print(f"Failed to send POST request. Status code: {response.status_code}")
    except requests.ConnectionError:
        print(f"Failed to connect to the server.")

if __name__ == "__main__":
    send_post_request()
