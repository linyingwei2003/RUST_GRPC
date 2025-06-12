use grpc_demo_proto::ads::{ads_service_server::{AdsService, AdsServiceServer}, AdRequest, AdResponse, ShopCampaign};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

#[derive(Clone)]
struct CampaignState {
    campaigns: Arc<Mutex<HashMap<String, Campaign>>>,
}

#[derive(Debug, Clone)]
struct Campaign {
    settings: ShopCampaign,
    spent: f64,
}

impl Campaign {
    fn new(settings: ShopCampaign) -> Self {
        Self { settings, spent: 0.0 }
    }

    fn remaining_budget(&self) -> f64 {
        self.settings.budget - self.spent
    }

    fn next_bid(&self) -> f64 {
        let target = self.settings.roas * self.settings.acquire_cost;
        self.remaining_budget().min(target)
    }

    fn record_bid(&mut self, amount: f64) {
        self.spent += amount;
    }
}

#[derive(Clone)]
struct AdsServer {
    state: CampaignState,
}

#[tonic::async_trait]
impl AdsService for AdsServer {
    async fn serve_ad(&self, request: Request<AdRequest>) -> Result<Response<AdResponse>, Status> {
        let id = request.into_inner().campaign_id;
        let mut campaigns = self.state.campaigns.lock().unwrap();
        let campaign = campaigns.get_mut(&id).ok_or_else(|| Status::not_found("campaign not found"))?;
        let bid = campaign.next_bid();
        campaign.record_bid(bid);
        let reply = AdResponse { ad_text: format!("Ad for campaign {} on Shop", id), bid };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let mut initial = HashMap::new();
    initial.insert(
        "shop_campaign".to_string(),
        Campaign::new(ShopCampaign { id: "shop_campaign".to_string(), budget: 1000.0, roas: 2.0, acquire_cost: 5.0 })
    );

    let ads_server = AdsServer { state: CampaignState { campaigns: Arc::new(Mutex::new(initial)) } };

    let addr = "[::]:50055".parse()?;
    info!("Ads server listening on {}", addr);

    Server::builder()
        .add_service(AdsServiceServer::new(ads_server))
        .serve(addr)
        .await?;

    Ok(())
}
