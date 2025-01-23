import json
import os
from http.server import BaseHTTPRequestHandler, HTTPServer
from xml.etree.ElementTree import Element, SubElement, tostring

import boto3
from botocore.config import Config


sts_client = boto3.client(
    "sts",
    endpoint_url=os.environ.get("MINIO_TENANT_ENDPOINT"),
    aws_access_key_id=os.environ.get("MINIO_TENANT_ROOT_USER"),
    aws_secret_access_key=os.environ.get("MINIO_TENANT_ROOT_PASSWORD"),
    config=Config(region_name="ap-northeast-1", signature_version="v4"),
)


class FakeMinioStsOperator(BaseHTTPRequestHandler):
    def do_POST(self):
        bucket_name = os.environ.get("CHLORIA_ORIGIN_BUCKET_NAME")
        # Ref: https://github.com/metalwhale/wave/blob/main/projects/chloria-minio/base/ocean/configmap.yaml
        policy = {
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Action": [
                        "s3:*",
                    ],
                    "Resource": [
                        f"arn:aws:s3:::{bucket_name}",
                        f"arn:aws:s3:::{bucket_name}/*",
                    ],
                },
            ],
        }
        # Doc: https://github.com/minio/minio/blob/RELEASE.2024-10-02T17-50-41Z/docs/sts/assume-role.md#sample-post-request
        credentials = sts_client.assume_role(
            RoleArn="arn:xxx:xxx:xxx:xxxx",
            RoleSessionName="anything",
            Policy=json.dumps(policy),
        )["Credentials"]
        # Doc: https://github.com/minio/minio/blob/RELEASE.2024-10-02T17-50-41Z/docs/sts/web-identity.md#sample-response
        response = Element("AssumeRoleWithWebIdentityResponse")
        result_element = SubElement(response, "AssumeRoleWithWebIdentityResult")
        credentials_element = SubElement(result_element, "Credentials")
        access_key_id_element = SubElement(credentials_element, "AccessKeyId")
        access_key_id_element.text = credentials["AccessKeyId"]
        secret_access_key_element = SubElement(credentials_element, "SecretAccessKey")
        secret_access_key_element.text = credentials["SecretAccessKey"]
        session_token_element = SubElement(credentials_element, "SessionToken")
        session_token_element.text = credentials["SessionToken"]
        expiration_element = SubElement(credentials_element, "Expiration")
        expiration_element.text = credentials["Expiration"].isoformat()
        self.send_response(200)
        self.send_header("Content-type", "text/xml")
        self.end_headers()
        self.wfile.write(tostring(response))


def run(server_class=HTTPServer, handler_class=FakeMinioStsOperator):
    # Same port with MinIO STS service
    # Ref: https://github.com/minio/operator/blob/v6.0.4/helm/operator/templates/sts-service.yaml#L10
    server_address = ("", 4223)
    httpd = server_class(server_address, handler_class)
    httpd.serve_forever()


if __name__ == "__main__":
    run()
