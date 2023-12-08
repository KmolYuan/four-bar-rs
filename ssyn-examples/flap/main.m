clear;clc;close all
wt = linspace(0, 2*pi, 60);
phi = 0.4 * sin(wt) + 0.5;
psi = 0.36 * cos(wt) - 0.3;
theta = 0.25 * cos(wt) + 0.2;
p = [-10; 1; 0; 1];
coord = zeros(60, 3);
for i = 1:60
    m = DHm(0, pi/2, 0, phi(i)) * DHm(0, pi/2, 0.015, psi(i) - pi/2) * DHm(0, pi/2, 0, theta(i));
    p_new = m * p;
    coord(i, :) = p_new(1:3)';
end
writematrix(coord, "flap.closed.csv")

figure
plot3(coord(:,1),coord(:,2),coord(:,3),'ro')
xlabel('x')
ylabel('y')
zlabel('z')
cameratoolbar('SetCoordSys','y','setmode','orbit')
rotate3d
