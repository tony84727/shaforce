@group(0)
@binding(0)
var<storage, read> chunks: array<u32>;
@group(0)
@binding(1)
var<storage, read_write> output: array<u32, 5>;

fn rotate_left(input: u32, count: u32) -> u32 {
	return (input << count) | (input >> (32u - count));
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
	output[0] = 0x67452301u;
	output[1] = 0xEFCDAB89u; 
	output[2] = 0x98BADCFEu;
	output[3] = 0x10325476u;
	output[4] = 0xC3D2E1F0u;
	for(var i = 0u; i < arrayLength(&chunks); i += 16u) {
		var block_hash: array<u32, 5>;
		for (var i = 0u; i < 5u; i++) {
			block_hash[i] = output[i];
		}
		var message: array<u32, 80>;
		for (var j = 0u; j < 16u; j++) {
			message[j] = chunks[i + j];
		}
		for (var j = 16u; j < 80u; j++) {
			message[j] = rotate_left(message[j - 3u] ^ message[j - 8u] ^ message[j - 14u] ^ message[j - 16u],1u);
		}
		for (var k = 0u; k < 80u; k++) {
			var f: u32 = 0u;
			var h: u32 = 0u;
			if (k >= 0u && k <= 19u) {
				f = (block_hash[1u] & block_hash[2u]) | (~block_hash[1u] & block_hash[3u]);
				h = 0x5A827999u;
			} else if (k >= 20u && k <= 39u) {
				f = block_hash[1] ^ block_hash[2] ^ block_hash[3];
				h = 0x6ED9EBA1u;
			} else if (k >= 40u && k <= 59u) {
				f = (block_hash[1u] & block_hash[2u]) | (block_hash[1u] & block_hash[3u]) | (block_hash[2u] & block_hash[3u]);
				h = 0x8F1BBCDCu;
			} else {
				f = block_hash[1u] ^ block_hash[2u] ^ block_hash[3u];
				h = 0xCA62C1D6u;
			}
			var temp = rotate_left(block_hash[0u], 5u) + f + block_hash[4u] + h + message[k];
			for (var r = 1u; r < 5u; r++) {
				block_hash[r] = block_hash[r - 1u];
			}
			block_hash[2u] = rotate_left(block_hash[2u], 30u);
			block_hash[0u] = temp;
		}
		for (var r = 0u; r < 5u; r++) {
			output[r] += block_hash[r];
		}
	}
}
