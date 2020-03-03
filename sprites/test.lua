height = 64
width = 64

position = {["x"] = 0, ["y"] = 0}
velocity = {["x"] = 1, ["y"] = 1}

function init()
	return {"resources/missing.png"}, width, height
end

function tick(w, h)
	position.x = position.x + velocity.x
	position.y = position.y + velocity.y
	if position.x + width == w or position.x == 0 then
		velocity.x = velocity.x * -1
	end
	if position.y + height == h or position.y == 0 then
		velocity.y = velocity.y * -1
	end
	return 0, position.x, position.y
end
