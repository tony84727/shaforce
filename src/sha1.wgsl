@group(0)
@binding(0)
var<storage, read> chunks: array<u32>;
@group(0)
@binding(1)
var<storage, read_write> output: array<u32, 5>;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
	output[0] = 0x67452301u;
	output[1] = 0xEFCDAB89u; 
	output[2] = 0x98BADCFEu;
	output[3] = 0x10325476u;
	output[4] = 0xC3D2E1F0u;
	for(var i = 0; i < arrayLength(&chunks); i += 16) {
		var message: array<u32, 80>;
		for (var j = 0; j < 16; j++) {
			message[j] = chunks[i + j];
		}
		for (var j = 16; j < 80; j++) {
			message[j] = message[j - 3] ^ message[j - 8] ^ message[j - 16] << 1;
		}
		for (var k = 0; k < 80; k++) {
			var block_hash: array<u32, 5> = output;
			var f: u32 = 0;
			var h: u32 = 0;
			if (h >= 0 && h <= 19) {
				f = (block_hash[1] & block_hash[2]) | (!block_hash[1] & block_hash[3]);
				h = 0x5A827999u;
			} else if (h >= 20 && h <= 39) {
				f = block_hash[1] ^ block_hash[2] ^ block_hash[3];
				h = 0x6ED9EBA1u;
			} else if (h >= 40 && h <= 59) {
				f = (block_hash[1] & block_hash[2]) | (block_hash[1] & block_hash[3]) | (block_hash[2] & block_hash[3]);
				h = 0x8F1BBCDCu;
			} else {
				f = block_hash[1] ^ block_hash[2] ^ block_hash[3];
				k = 0xCA62C1D6u;
			}
			var temp = block_hash[0] << 5 + f + block_hash[4] + h + message[k];
			for (var r = 1; r < 5; r++) {
				block_hash[r] = block_hash[r-1];
			}
			block_hash[2] = block_hash[2] << 30;
			block_hash[0] = temp;
			for (var r = 0; r < 5; r++) {
				output[r] += block_hash[r];
			}
		}
	}
}
