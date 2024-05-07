$(document).ready(function() {
    function fetchAndDisplayAuctions() {
        $.getJSON('/api/auctions', function(data) {
            var auctions = data.auctions;
            var auctionHistoryHtml = '';
            var followingItemsHtml = '';
            var openAuctionsHtml = '';

            Object.keys(auctions).forEach(function(key) {
                var auction = auctions[key];
                var actionButtonHtml = '<td><button class="bid-button fa fa-money" onclick="showBidModal()"></button></td>';
                var rowHtml = '<tr>' +
                    '<td>' + auction.item_name + '</td>' +
                    '<td>$' + (auction.bids.length > 0 ? auction.bids[auction.bids.length - 1].amount : auction.starting_bid) + '</td>' +
                    '<td>' + new Date(auction.end_time * 1000).toLocaleString() + '</td>' +
                    '<td>' + (auction.active ? 'Active' : 'Closed') + '</td>';

                if (!auction.active) {
                    auctionHistoryHtml += rowHtml + '</tr>';
                } else {
                    openAuctionsHtml += rowHtml + actionButtonHtml + '</tr>';
                    if (auction.following) { // Check if auction is followed
                        followingItemsHtml += rowHtml + actionButtonHtml + '</tr>';
                    } else {
                        followingItemsHtml += rowHtml + '<td></td></tr>'; // No action button if not followed
                    }
                }
            });

            $('#auctionHistoryTable tbody').html(auctionHistoryHtml);
            $('#followingItemsTable tbody').html(followingItemsHtml);
            $('#openAuctionsTable tbody').html(openAuctionsHtml);
        });
    }

    fetchAndDisplayAuctions();
    setInterval(fetchAndDisplayAuctions, 30000); // Refresh data every 30 seconds

    window.showBidModal = function() {
        $('#bidModal').modal('show');
    }
});