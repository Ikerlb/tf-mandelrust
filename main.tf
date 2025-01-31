terraform {
 required_providers {
   aws = {
     source  = "hashicorp/aws"
     version = "4.13.0"
   }
 }
}
 
provider "aws" {
 region = "us-east-1"
}

resource "aws_iam_role" "mandelrust-iam-role" {
  name = "mandelrust-iam-role"
  assume_role_policy = jsonencode(
		{
			Statement = [
				{
					Action    = "sts:AssumeRole"
					Effect    = "Allow"
					Principal = {
						Service   = "lambda.amazonaws.com"
					}
				},
			]
			Version   = "2012-10-17"
		}
	)
} 

resource "aws_lambda_function_url" "mandelrust-url" {
  function_name      = aws_lambda_function.mandelrust.function_name
  authorization_type = "NONE"
}

data "archive_file" "lambda_zip" {
  type        = "zip"
  source_file  = "resources/target/x86_64-unknown-linux-musl/release/bootstrap"
  output_path = "resources/target/x86_64-unknown-linux-musl/release/lambda.zip"
}

resource "aws_lambda_function" "mandelrust" {
  description = "Deploying a Rust function on Lambda"
  layers = [] 
  function_name = "terraform-mandelrust-lambda"
  role = aws_iam_role.mandelrust-iam-role.arn
  architectures = ["x86_64"]
  memory_size = 1024
  timeout = 10
  runtime = "provided.al2"
  handler = "not.required"
  filename = "resources/target/x86_64-unknown-linux-musl/release/lambda.zip"
  source_code_hash = data.archive_file.lambda_zip.output_base64sha256
}
