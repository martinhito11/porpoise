![Welcome to Porpoise](https://assets.iflscience.com/assets/articleNo/73290/aImg/74736/smiling-porpoise-o.webp)

Porpoise is an online LLM web application. The basic premise of the application is that it implements a single endpoint, the equivalent of OpenAI's chat/completion POST request https://platform.openai.com/docs/api-reference/chat .

The service works as follows:

1. Receive a user's query 
2. Send the query to OpenAI with a pre-generated prompt to decompose the query into simple words that are good for Googling 
3. Search Google with that query using Google's Custom Search API, retrieiving the top 10 results 
4. Parse the URLs of those results, removing those from blacklisted sites (sites where GET requests are blocked), then do a GET on them to retrieve the raw HTML.
5. Scrape and clean the HTML with Rust regex libraries. Send the cleaned up HTML to OpenAI in parallel to clean it further. 
6. Put together a final prompt using the scraped information, the associated URLs with the scraped information, and send it to OpenAI with a pre-generated prompt asking it to answer the question using both the given information, any pre-trained information it has, and to cite any information it uses from the given using the provided URLs.


The app is currently hosted here: https://porpoise-hyuqtm35xq-ue.a.run.app

Test it using the provided poster.py script or using this cURL command:
```
curl -X POST 'https://porpoise-hyuqtm35xq-ue.a.run.app/chat/completions' \
-H 'Content-Type: application/json' \
-d '{
    "model": "gpt-4-turbo",
    "messages": [
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "reddit ipo news"}
    ]
}'
```

the above request should give a response that looks like this:
```
{
  "id": "chatcmpl-9DDlHGfUqN2W1NRwsv15MvgIpxoXI",
  "object": "chat.completion",
  "created": 1712938191,
  "model": "gpt-4-turbo-2024-04-09",
  "system_fingerprint": "fp_76f018034d",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Reddit is preparing to launch an initial public offering (IPO) 
        valued at up to $6.4 billion, aiming to raise almost $750 million by offering 
        22 million shares priced between $31 and $34 each. This significant step 
        includes not only institutional investors but also an unconventional move to 
        offer shares directly to Reddit's active users and moderators. This approach 
        integrates these central community figures into the financial evolution of the 
        company, marking a novel method in handling tech IPOs. This participatory strategy 
        could add some variability to the IPO process due to the immediate selling rights 
        granted to these users, potentially affecting stock price stability post-IPO. 
        Additionally, Reddit's deal with Google, involving access to its extensive data 
        for AI training, represents a new revenue stream while its persistent unprofitability 
        raises critical considerations for potential investors ([Reddit looking to raise 
        almost $750 million in IPO | AP News](https://www.apnews.com/reddit-ipo-details-and-strategy))."
      },
      "logprobs": null,
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 8679,
    "completion_tokens": 192,
    "total_tokens": 8871
  }
}
```
Interesting problems I came across: 


Things I did: 
- Parallelized the OpenAI requests for cleaning up the HTML using tokio
- I found a third party API handler for SERP data called serpstack https://serpstack.com/ . I tried using this instead of the google custom search API, but it was useless to me because it didn't summarize results.

Left to try: 
- Add perplexity 
- Add streaming 
- Add Google's provided "snippet" from its custom search API to see if that provides verbose enough data. It would make for a faster endpoint if we just used the snippet rather than scraping raw HTML