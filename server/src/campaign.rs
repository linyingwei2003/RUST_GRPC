/// Campaign struct representing an advertising campaign with bidding logic
#[derive(Debug)]
pub struct Campaign {
    pub remaining_budget: f64,
    pub roas: f64,
}

impl Campaign {
    /// Creates a new Campaign instance
    pub fn new(remaining_budget: f64, roas: f64) -> Self {
        Self {
            remaining_budget,
            roas,
        }
    }

    /// Calculates the next bid amount based on expected revenue divided by ROAS
    /// Returns the minimum of remaining budget and calculated max bid
    pub fn next_bid(&self) -> f64 {
        // Mock value for expected revenue - in the future this will be provided by an ML model
        const EXPECTED_REVENUE: f64 = 12.0;
        
        // Calculate max bid using expected revenue divided by ROAS
        let max_bid = if self.roas > 0.0 {
            EXPECTED_REVENUE / self.roas
        } else {
            0.0 // Handle division by zero case
        };
        
        // Return the minimum of remaining budget and calculated max bid
        self.remaining_budget.min(max_bid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_campaign_creation() {
        let campaign = Campaign::new(100.0, 2.0);
        assert_eq!(campaign.remaining_budget, 100.0);
        assert_eq!(campaign.roas, 2.0);
    }

    #[test]
    fn test_next_bid_basic_calculation() {
        let campaign = Campaign::new(100.0, 2.0);
        let bid = campaign.next_bid();
        // Expected: 12.0 / 2.0 = 6.0, min(100.0, 6.0) = 6.0
        assert_eq!(bid, 6.0);
    }

    #[test]
    fn test_next_bid_budget_limited() {
        let campaign = Campaign::new(3.0, 2.0);
        let bid = campaign.next_bid();
        // Expected: 12.0 / 2.0 = 6.0, min(3.0, 6.0) = 3.0
        assert_eq!(bid, 3.0);
    }

    #[test]
    fn test_next_bid_high_roas() {
        let campaign = Campaign::new(100.0, 4.0);
        let bid = campaign.next_bid();
        // Expected: 12.0 / 4.0 = 3.0, min(100.0, 3.0) = 3.0
        assert_eq!(bid, 3.0);
    }

    #[test]
    fn test_next_bid_zero_roas() {
        let campaign = Campaign::new(100.0, 0.0);
        let bid = campaign.next_bid();
        // Expected: 0.0 (division by zero protection)
        assert_eq!(bid, 0.0);
    }

    #[test]
    fn test_next_bid_very_small_roas() {
        let campaign = Campaign::new(100.0, 0.5);
        let bid = campaign.next_bid();
        // Expected: 12.0 / 0.5 = 24.0, min(100.0, 24.0) = 24.0
        assert_eq!(bid, 24.0);
    }

    #[test]
    fn test_next_bid_zero_budget() {
        let campaign = Campaign::new(0.0, 2.0);
        let bid = campaign.next_bid();
        // Expected: 12.0 / 2.0 = 6.0, min(0.0, 6.0) = 0.0
        assert_eq!(bid, 0.0);
    }
}