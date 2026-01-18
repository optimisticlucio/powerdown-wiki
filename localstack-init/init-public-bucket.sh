#!/bin/bash
awslocal s3 mb s3://powerdown-public-storage
awslocal s3api put-bucket-cors --bucket powerdown-public-storage --cors-configuration '{
    "CORSRules": [
        {
            "AllowedHeaders": ["*"],
            "AllowedMethods": ["GET", "PUT", "POST"],
            "AllowedOrigins": ["*"],
            "ExposeHeaders": [],
            "MaxAgeSeconds": 3000
        }
    ]
}'
awslocal s3api put-bucket-lifecycle-configuration --bucket powerdown-public-storage --lifecycle-configuration '{
    "Rules": [
        {
            "ID": "ExpireTempFolder",
            "Filter": {
                "Prefix": "temp/"
            },
            "Status": "Enabled",
            "Expiration": {
                "Days": 1
            }
        }
    ]
}'
echo "S3 buckets created"