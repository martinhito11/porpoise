use std::error::Error;

use tokio::time::{timeout, Duration};

use crate::api_dtos::{
    ChatCompletionRequestMessage, 
    ChatCompletionRequest, 
    ChatCompletionResponse, 
    ChatCompletionChoice, 
    ChatCompletionMessage, 
    ChatCompletionUsage,
    Role,
};

pub async fn send_chat_completion(req: ChatCompletionRequest, internal_req: bool) -> Result<ChatCompletionResponse, Box<dyn Error>> {
    println!("Received request to send to OpenAI");
    let api_key = "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";
    let url = "https://api.openai.com/v1/chat/completions";
    
    let payload = ChatCompletionRequest { model: req.model.to_string(), messages: req.messages };

    // Send the request
    let client = reqwest::Client::new();

    let response = timeout(Duration::from_secs(10), async move {
        let resp = client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key.to_string()))
            .body(serde_json::to_string(&payload)?)
            .send()
            .await?;

        match resp.status() {
            reqwest::StatusCode::OK => {
                let resp_text = resp.text().await?;
                let deserialized_resp: Result<ChatCompletionResponse, Box<dyn Error>> = serde_json::from_str(&resp_text).map_err(|e| Box::new(e) as Box<dyn Error>);
                return deserialized_resp
            }
            _ => {
                eprintln!("Request failed with status code: {}", resp.status());
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Request failed")));
            }
        }
    }).await?;

    match response {
        Ok(result) => Ok(result),
        Err(err) => {
            if internal_req {
                eprintln!("Operation timed out");
                Ok(ChatCompletionResponse {
                    id: "dummy_id".to_string(),
                    object: "chat.completion".to_string(),
                    created: 0,
                    model: DEFAULT_MODEL.to_string(),
                    system_fingerprint: "dummy_fingerprint".to_string(),
                    choices: vec![
                        ChatCompletionChoice {
                            index: 0,
                            message: ChatCompletionMessage {
                                role: "Assistant".to_string(),
                                content: "".to_string(), // Empty string for the message
                            },
                            logprobs: (),
                            finish_reason: "dummy_finish_reason".to_string()
                        },
                    ],
                    usage: ChatCompletionUsage {
                        prompt_tokens: 0, 
                        completion_tokens: 0,
                        total_tokens: 0,
                    },
                })
            }
            else {
                Err(err)
            }
        }
    }
}

pub async fn get_googleable_query(user_msg: &str) -> String {
    let mut query_str = DECOMPOSE_QUERY_STR.to_string();
    query_str.push_str(" \n");
    query_str.push_str(&user_msg.to_string());

    let req_message_system: ChatCompletionRequestMessage = ChatCompletionRequestMessage {
        role: Role::System,
        content: GOOGLEABLE_SYSTEM_STR.to_string()
    };
    let req_message_user: ChatCompletionRequestMessage = ChatCompletionRequestMessage {
        role: Role::User,
        content: query_str
    };
    let req: ChatCompletionRequest = ChatCompletionRequest {
        model: DEFAULT_MODEL.to_string(),
        messages: vec![req_message_system, req_message_user]
    };
    match send_chat_completion(req, true).await {
        Ok(query) => query.choices[0].message.content.clone(),
        Err(_) => "".to_string(),
    }
}

pub const DEFAULT_MODEL: &str = "gpt-4-turbo";

pub const WITH_INFO_USER_QUERY_STR: &str = 
"
Answer the following query using the above information and your own pre-trained dataset. 
Using the provided URL above each paragraph, cite the URL explicitly in your response, imbedded inside 
the response as you write it. 
Make no mention of the fact that you are pulling information from the above paragraphs and your own pre-trained dataset.

For example, a good response looks like:
Some of the best restaurants in Chicago include Community Tavern, Smoque BBQ, Superkhana International, 
Pat's Pizza and Ristorante, Mi Tocaya Antojería, Lula Cafe, Taqueria Chingon, Redhot Ranch, Hermosa Restaurant, 
Alinea, Oriole, Johnnie's Beef, Birrieria Zaragoza, Kasama, Kyōten, Jeong, The Publican, S.K.Y., Virtue, Maple & Ash, 
Vito and Nick's Pizzeria, Boka, Monteverde, Smyth + The Loyalist, Boonie's Filipino Restaurant, Gene & Jude's, HaiSous 
Vietnamese Kitchen, Bavette’s, and The Omakase Room at Sushi-san. These restaurants offer a wide variety of culinary 
experiences, from fine dining to casual eats, showcasing the diverse and vibrant food scene in the city of Chicago [https://chicago.eater.com/maps/38-best-restaurants-in-chicago]. 


";

pub const GOOGLEABLE_SYSTEM_STR: &str = 
"
You are an assistant whose job is to follow user prompting exactly, providing concise responses within word limits.
";

pub const DECOMPOSE_QUERY_STR: &str = 
"
I want you to decompose a search query I give you into googleable terms in order to fill in potential 
gaps in information that a trained LLM could have for up to date data. 
For example, I will provide you with each query below, where you will respond with the decomposed text below the query.

How effective is the new COVID-19 vaccine by BioNTech?
effectiveness BioNTech COVID-19 vaccine

Latest developments in artificial intelligence?
latest developments artificial intelligence

What's the current situation with the Amazon rainforest fires?
current situation Amazon rainforest fires

Upcoming SpaceX launch details?
upcoming SpaceX launch details

Trends in renewable energy 2024?
renewable energy trends 2024

Who won the Nobel Prize in Physics this year?
Nobel Prize Physics winner 2024

What are the latest tech IPOs?
latest tech IPOs

Recent advancements in quantum computing?
recent advancements quantum computing

What are the global impacts of the recent economic policy changes in the EU?
global impacts economic policy changes EU

Current top-rated smartphones in the market?
current top-rated smartphones

Who is Elon Musk?
elon musk

What happened in the world last week that people got excited about?
top news stories last week
";

pub const CLEAN_HTML_BODY_QUERY_STR: &str = 
"
I want you to clean up a paragraph of parsed HTML data  to remove all extraneous, non-relevant information for a given query on that data. I want you to respond with only the cleaned up data, and no reference to this query or the clean up of the data. For example, I will provide you with the text and query below, and you will respond with the cleaned up text.

Data:
Tesla Model Y was the best-selling car worldwide in the first quarter&lt;img alt class=D(n) src=/_td_api/beacon/info?beaconType=noJSenabled&amp;bucket=finance-US-en-US-def&amp;code=pageRender&amp;device=desktop&amp;lang=en-US&amp;pageName=deeplink&amp;region=US&amp;rid=6cld0m1j1h25o&amp;site=finance&amp;t=1712883896869&gt; HOME MAIL NEWS FINANCE SPORTS ENTERTAINMENT LIFE SEARCH SHOPPING YAHOO PLUS MORE... Yahoo Finance Sign in Mail Sign in to view your mail Finance Watchlists My Portfolio Markets YF Chartbook Calendars Trending Tickers Stocks: Most Actives Stocks: Gainers Stocks: Losers Top ETFs Futures World Indices Currencies Top Mutual Funds Options: Highest Open Interest Options: Highest Implied Volatility US Treasury Bonds Rates Currency Converter News Latest News From the Newsroom Stock Market News Earnings Politics Economic News Morning Brief Personal Finance News Crypto News Bidenomics Report Card Videos Yahoo Finance Invest Yahoo Finance Live ETF Report Options 101 Good Buy or Goodbye FA Corner Yahoo Finance Plus Dashboard Research Reports Investment Ideas Community Insights Webinars Blog Screeners Saved Screeners Equity Screener Mutual Fund Screener ETF Screener Futures Screener Index Screener Analyst Rating Screener Technical Events Screener Smart Money Screener Top Holdings Screener Personal Finance Credit Cards Balance transfer cards Cash-back cards Travel cards Rewards cards Banking Personal loans Student loans Insurance Car insurance Mortgages Taxes Crypto Sectors Basic Materials Communication Services Consumer Cyclical Consumer Defensive Energy Financial Services Healthcare Industrials Real Estate Technology Utilities Contact Us … AdvertisementU.S. markets closedS&amp;P Futures5,246.00+2.75 (+0.05%)&nbsp;Dow Futures38,772.00+40.00 (+0.10%)&nbsp;Nasdaq Futures18,493.75+8.75 (+0.05%)&nbsp;Russell 2000 Futures2,057.50-0.40 (-0.02%)&nbsp;Crude Oil85.51+0.49 (+0.58%)&nbsp;Gold2,395.10+22.40 (+0.94%)&nbsp;Silver28.50+0.25 (+0.90%)&nbsp;EUR/USD1.07310.0000 (-0.00%)&nbsp;10-Yr Bond4.5760+0.0160 (+0.35%)&nbsp;Vix14.91-0.89 (-5.63%)&nbsp;GBP/USD1.2558+0.0002 (+0.02%)&nbsp;USD/JPY152.9950-0.2080 (-0.14%)&nbsp;Bitcoin USD70,307.53-200.41 (-0.28%)&nbsp;CMC Crypto 200885.540.00 (0.00%)&nbsp;FTSE 1007,923.80-37.41 (-0.47%)&nbsp;Nikkei 22539,634.01+191.38 (+0.49%)&nbsp;Yahoo FinanceTesla Model Y was the best-selling car worldwide in the first quarterIt's also the first time a pure EV topped the global sales rankings.Read full article227Oops!Something went wrong.Please try again later.More content belowOops!Something went wrong.Please try again later.More content belowOops!Something went wrong.Please try again later.More content belowTSLATMTOYOFPras Subramanian·Senior ReporterUpdated May 30, 2023 at 11:49 AM·3 min readIn this article: Oops!Something went wrong.Please try again later.More content belowOops!Something went wrong.Please try again later.More content belowOops!Something went wrong.Please try again later.More content belowTSLAWatchlistTMTOYOFTesla (TSLA) scored a huge milestone in the first quarter amid its crusade to ramp up global EV adoption — producing the best-selling car in the world.But the electric vehicle maker may have paid a big price with lower profit margins.A new report from data firm JATO Dynamics, along with automotive site Motor.1.com, found the Tesla Model Y was the No. 1 selling vehicle globally in the first quarter of the year, marking the first time an EV was the top-selling vehicle.JATO said Tesla sold 267,200 Model Ys in Q1, up 69% from a year ago. The second-best-selling car was Toyota’s Corolla, with 256,400 vehicles sold globally. JATO’s data spanned 53 international markets, plus data and forecasts for 31 other markets and estimates for the balance of global markets.For Tesla, it could be the beginning of a new sales trend for the Model Y, which might see it being the global leader in sales for 2023.&lt;iframe src=https://flo.uri.sh/visualisation/13954403/embed?auto=1&gt;&lt;/iframe&gt;“It seems that Tesla has the wind in its sails because the Model Y is an SUV and it is electric, a win-win combination at the moment,” JATO industry specialist Felipe Munoz said in the report published on Motor1.com. “On the other hand, the Corolla has the advantage of being a truly global product, being available in almost every country in the world. This makes it less vulnerable to any geopolitical clashes between China and the United States, for example.”Drilling deeper into the data, JATO found that China accounted for 35% of all Model Y sales, with the US close behind at 31%. For those two countries, Model Y sales grew 26% in China and a whopping 68% in the US versus a year ago. The Tesla Model Y was also the top-selling vehicle in the EU.Of the top 5 selling vehicles, the other four spots are occupied by Toyota vehicles — the Corolla, Hilux pickup, RAV4/Wildlander CUV, and Camry sedan.Even as Tesla expands its footprint and aims to release a cheaper gen-3 vehicle, the company is paying a cost to grow its Model Y sales.Tesla instituted a number of price cuts in the US, Asia, and some European markets in Q1 of this year; in its earnings statement, the company reported gross margins dipped to 19.3%, reflecting the costs of those price cuts.Story continuesAccording to data compiled by Yahoo Finance, Tesla’s Model Y Long Range started the year with a $65,990 MSRP. The version now has a starting price of $50,490, a drop of $15,500, or 23.4% from the start of the year.While a decrease in gross margins is a significant worry for investors, CEO Elon Musk says he isn’t concerned, because it’s all part of his vision for the company.&lt;img alt=\"Tesla US Model Y order page as of 5/30/2023\" src=https://s.yimg.com/ny/api/res/1.2/XuQ_K0SXaeEYGo2JCnHeAw--/YXBwaWQ9aGlnaGxhbmRlcjt3PTk2MA--/https://s.yimg.com/os/creatr-uploaded-images/2023-05/f33d8010-fef5-11ed-be3c-4fb40fc67b1b class=caas-img&gt;Tesla US Model Y order page as of 5/30/2023 (Tesla.com)“We've taken a view that pushing for higher volumes and a larger fleet is the right choice here versus a lower volume and higher margin,” Musk said during the company’s Q1 earnings call.Musk believes the company, over time, will be able to generate significant profit through services like FSD (full-self driving) and autopilot autonomy services. “We do believe we're like laying the groundwork here, and that it's better to ship a large number of cars at a lower margin, and subsequently, harvest that margin in the future as we perfect autonomy,” he said.Tesla investors and automotive industry experts will be keen to see if the trend of rising Model Y sales continues in Q2 with the federal EV tax credit available for the entire quarter in the US. However, the company may face some headwinds in China due to recent macroeconomic weakness.—Pras Subramanian is a reporter for Yahoo Finance. You can follow him on Twitter and on Instagram.Click here for the latest stock market news and in-depth analysis, including events that move stocksRead the latest financial and business news from Yahoo FinanceTRENDING 1. Oil prices head back up on Middle East jitters 2. Peru Passes Bill Allowing for $7 Billion in Pension Withdrawals 3. Singapore Keeps Monetary Policy Tight as Price Risks Linger 4. Asian Stocks Edge Higher as Tech Lifts US Equities: Markets Wrap 5. PRESS DIGEST-British Business - April 12Advertisement&lt;img src=https://sb.scorecardresearch.com/p?c1=2&amp;c2=7241469&amp;c5=1197809822&amp;c7=https%3A%2F%2Ffinance.yahoo.com%2Fnews%2Ftesla-model-y-was-the-best-selling-car-worldwide-in-the-first-quarter-154909234.html&amp;c14=-1&gt

Query: 
What's Tesla's most popular car

Cleaned up text:
In the first quarter, the Tesla Model Y emerged as the best-selling car worldwide. This marked the first time a fully electric vehicle (EV) topped the global sales rankings. According to JATO Dynamics, Tesla sold 267,200 Model Ys, surpassing the sales of Toyota’s Corolla, which sold 256,400 vehicles. The surge in sales was notably significant in China and the US, contributing 35% and 31% of the total Model Y sales respectively.

Tesla's strategy involved several price cuts across the US, Asia, and Europe, which affected the company's gross margins but was part of a broader plan to increase vehicle volume and market presence. Elon Musk, CEO of Tesla, emphasized that this approach aims to enhance future profitability through services like full-self driving and autopilot autonomy. The trend of rising Model Y sales is expected to continue, potentially boosted by the federal EV tax credit in the US.

Do the same with the following data and query pair:
";