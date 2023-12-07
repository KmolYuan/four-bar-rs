theta = linspace(30, 330, 40);

x = sind(65) * sind(-0.4685 * theta + 173.1994);
y = -cosd(theta) .* cosd(-0.4685 * theta + 173.1994) + cosd(65) * sind(theta) .* sind(-0.4685 * theta + 173.1994);
z = -sind(theta) .* cosd(-0.4685 * theta + 173.1994) - cosd(65) * cosd(theta) .* sind(-0.4685 * theta + 173.1994);
writematrix(1000 * [x; y; z]', "fish.open.csv")

figure
plot3(x, y, z)
% hold on
% plot3([0, 1, 0, 0, -1, 0, 0], [0, 0, 1, 0, 0, -1, 0], [0, 0, 0, 1, 0, 0, -1], "ro")
xlabel("X")
ylabel("Y")
zlabel("Z")
axis equal
