clc;clear

t = linspace(0, 2*pi, 12)';
c = [0.4, 0.4, 0.4] + [-0.51*cos(t)+0.29*sin(t), 0.51*cos(t)+0.29*sin(t), -0.59*sin(t)];
writematrix(c, "circle.partial.csv")

figure
sphere
hold on
plot3(c(:,1),c(:,2),c(:,3),'ro')
xlabel('x')
ylabel('y')
zlabel('z')
