local hello = package({
	name = 'hello',
	src = {},
	build = function(builder)
		builder:add_text_file('bin/hello', {
			[[
				#!/bin/sh
				echo "Hello World!"
      ]],
		})
	end,
})

-- This function returns a new package that contains all input packages.
-- This is the equivelent of a Linux initramfs at the moment.
local makeSystem = function(pkgs)
	local system = package({
		name = 'system-' .. os.date('%Y%m%d%H%M%S'),
		src = {},
		build = function(builder) end,
	})
end
