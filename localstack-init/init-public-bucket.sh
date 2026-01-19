#!/bin/bash
awslocal s3 mb s3://powerdown-public-storage
awslocal s3api put-bucket-cors --bucket powerdown-public-storage --cors-configuration '{
    "CORSRules": [
        {
            "AllowedHeaders": ["*"],
            "AllowedMethods": ["GET", "PUT", "POST"],
            "AllowedOrigins": ["*"],
            "ExposeHeaders": ["ETag", "x-amz-request-id", "x-amz-id-2"],
            "MaxAgeSeconds": 3000
        }
    ]
}'
echo "S3 buckets created"