import json
import requests  # To send captured data to your custom program

# header1 because headers=headers in req.post()
# confuses the shit out me !!
headers1 = {
        "Content-Type": "application/json"
}

class CustomMitmproxyAddon:
    def __init__(self):
        self.target_program_url = "http://127.0.0.1:5000/data"  # Replace with your program's URL

    def request(self, flow):
        """
        This method is triggered when a request is intercepted.
        """
        # Extract HTTP request details
        request_data = {
            "type": "request",
            "method": str(flow.request.method),
            "url": str(flow.request.url),
            "headers": str(dict(flow.request.headers)),
            "body": str(flow.request.text),
        }
        
        print(flow.request.text)

        # Send request data to the custom program
        self.send_to_program(request_data)

    def response(self, flow):
        """
        This method is triggered when a response is intercepted.
        """
        # Extract HTTP response details
        response_data = {
            "type": "response",
            "url": flow.request.url,
            "status_code": flow.response.status_code,
            "headers": dict(flow.response.headers),
            "body": flow.response.text,
        }
        print(flow.response.text)
        # Send response data to the custom program
        # self.send_to_program(response_data)

    def send_to_program(self, data):
        """
        Send data (request/response) to the custom program.
        """
        try:
            # Send the data to the target program
            
            # use data=json.dumps(data) with json header1 to stop main
            # rust server from bitching and doing its job correctly.
            response = requests.post(self.target_program_url, data=json.dumps(data),headers=headers1)

            print(f"Sent to program: {response.status_code} - {response.text}")
        except Exception as e:
            print(f"Error sending to custom program: {e}")

# Register the addon with mitmproxy
addons = [
    CustomMitmproxyAddon()
]

